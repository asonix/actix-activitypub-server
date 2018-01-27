use std::collections::BTreeMap;

use actix::{Actor, Address, AsyncContext, Context, Handler, SyncAddress};

use super::{Id, UserId, UsersId};
use super::posts::Posts;
use super::user::User;
use super::user::inbox::Inbox;
use super::user::outbox::Outbox;

pub mod messages;

use self::messages::*;

#[derive(Clone)]
pub struct UserAddress {
    user: SyncAddress<User>,
    inbox: SyncAddress<Inbox>,
    outbox: SyncAddress<Outbox>,
}

impl UserAddress {
    pub fn new(user_id: UserId, posts: SyncAddress<Posts>, users: SyncAddress<Users>) -> Self {
        let (user_local, user): (Address<_>, SyncAddress<_>) = User::new(user_id).start();

        let inbox = Inbox::new(user_local.clone()).start();
        let outbox = Outbox::new(user_id, user_local, posts, users).start();

        UserAddress {
            user,
            inbox,
            outbox,
        }
    }

    pub fn user(&self) -> &SyncAddress<User> {
        &self.user
    }

    pub fn inbox(&self) -> &SyncAddress<Inbox> {
        &self.inbox
    }

    pub fn outbox(&self) -> &SyncAddress<Outbox> {
        &self.outbox
    }
}

pub struct Users {
    users_id: UsersId,
    current_id: u64,
    users: BTreeMap<UserId, UserAddress>,
    posts: SyncAddress<Posts>,
    redundancy: Vec<SyncAddress<Users>>,
}

impl Users {
    pub fn new(users_id: UsersId, posts: SyncAddress<Posts>) -> Self {
        Users {
            users_id: users_id,
            current_id: 0,
            users: BTreeMap::new(),
            posts: posts,
            redundancy: Vec::new(),
        }
    }

    pub fn add_peer(mut self, peer: SyncAddress<Users>) -> Self {
        self.redundancy.push(peer);
        self
    }

    fn gen_next_id(&mut self) -> UserId {
        let id = Id(self.current_id);
        self.current_id += 1;
        UserId(self.users_id, id)
    }

    fn get_user(&self, user_id: UserId) -> Option<UserAddress> {
        self.users.get(&user_id).cloned()
    }

    fn get_users(&self, user_ids: Vec<UserId>) -> (Vec<UserAddress>, Vec<UserId>) {
        user_ids.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut addrs, mut user_ids), user_id| {
                if let Some(addr) = self.users.get(&user_id).cloned() {
                    addrs.push(addr);
                } else {
                    user_ids.push(user_id);
                }

                (addrs, user_ids)
            },
        )
    }

    fn add_user(&mut self, user_id: UserId, user_address: UserAddress) {
        self.users.insert(user_id, user_address);
    }

    fn new_user(&mut self, users: SyncAddress<Users>) -> (UserId, UserAddress) {
        let posts = self.posts.clone();
        let user_id = self.gen_next_id();
        let user_address = UserAddress::new(user_id, posts, users);

        self.add_user(user_id, user_address.clone());

        (user_id, user_address)
    }

    fn delete_user(&mut self, user_id: UserId) {
        self.users.remove(&user_id);
    }
}

impl Actor for Users {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if !self.redundancy.is_empty() {
            self.redundancy[0].send(RequestPeers(ctx.address()));
            self.redundancy[0].send(RequestBackfill(ctx.address(), 0));
        }
    }
}

impl Handler<Lookup> for Users {
    type Result = Result<UserAddress, ()>;

    fn handle(&mut self, msg: Lookup, _: &mut Context<Self>) -> Self::Result {
        self.get_user(msg.0).ok_or(())
    }
}

impl Handler<LookupMany> for Users {
    type Result = Result<(Vec<UserAddress>, Vec<UserId>), ()>;

    fn handle(&mut self, msg: LookupMany, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_users(msg.0))
    }
}

impl Handler<NewUser> for Users {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, _: NewUser, ctx: &mut Context<Self>) -> Self::Result {
        let (user_id, user_address) = self.new_user(ctx.address());

        for addr in &self.redundancy {
            addr.send(AnnounceNewUser(user_id, user_address.clone()));
        }

        Ok(user_id)
    }
}

impl Handler<AnnounceNewUser> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceNewUser, _: &mut Context<Self>) -> Self::Result {
        self.add_user(msg.0, msg.1);
    }
}

impl Handler<AnnounceDeleteUser> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceDeleteUser, _: &mut Context<Self>) -> Self::Result {
        self.delete_user(msg.0);
    }
}

impl Handler<AnnounceRedundancy> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceRedundancy, _: &mut Context<Self>) -> Self::Result {
        self.redundancy.push(msg.0);
    }
}

impl Handler<RequestPeers> for Users {
    type Result = ();

    fn handle(&mut self, msg: RequestPeers, _: &mut Context<Self>) -> Self::Result {
        msg.0.send(ReplyPeers(self.redundancy.clone()));

        self.redundancy.push(msg.0);
    }
}

impl Handler<ReplyPeers> for Users {
    type Result = ();

    fn handle(&mut self, msg: ReplyPeers, _: &mut Context<Self>) -> Self::Result {
        self.redundancy.extend(msg.0);
    }
}

impl Handler<RequestBackfill> for Users {
    type Result = ();

    fn handle(&mut self, msg: RequestBackfill, _: &mut Context<Self>) -> Self::Result {
        let backfill = self.users
            .iter()
            .skip(msg.1)
            .take(100)
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();

        msg.0.send(ReplyBackfill(msg.1, backfill));
    }
}

impl Handler<ReplyBackfill> for Users {
    type Result = ();

    fn handle(&mut self, msg: ReplyBackfill, ctx: &mut Context<Self>) -> Self::Result {
        if msg.1.len() == 100 {
            self.redundancy
                .get(0)
                .map(|node| node.send(RequestBackfill(ctx.address(), msg.0 + 100)));
        }

        self.users.extend(msg.1);
    }
}

impl Handler<PeerSize> for Users {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: PeerSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.redundancy.len())
    }
}

impl Handler<UserSize> for Users {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: UserSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.users.len())
    }
}
