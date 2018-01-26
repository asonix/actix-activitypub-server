use std::collections::HashMap;

use actix::{Actor, Address, AsyncContext, Context, Handler, SyncAddress};

use super::{Id, UserId};
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
    current_id: u64,
    users: HashMap<UserId, UserAddress>,
    posts: SyncAddress<Posts>,
    redundancy: Vec<SyncAddress<Users>>,
}

impl Users {
    pub fn new(posts: SyncAddress<Posts>) -> Self {
        Users {
            current_id: 0,
            users: HashMap::new(),
            posts: posts,
            redundancy: Vec::new(),
        }
    }

    pub fn new_user(&mut self, posts: SyncAddress<Posts>, users: SyncAddress<Users>) -> (UserId, UserAddress) {
        let user_id = self.gen_next_id();
        let user_address = UserAddress::new(user_id, posts, users);

        self.users.insert(user_id, user_address.clone());

        (user_id, user_address)
    }

    fn gen_next_id(&mut self) -> UserId {
        let id = self.current_id;
        self.current_id += 1;
        Id(id)
    }
}

impl Actor for Users {
    type Context = Context<Self>;
}

impl Handler<Lookup> for Users {
    type Result = Result<UserAddress, ()>;

    fn handle(&mut self, lookup: Lookup, _: &mut Context<Self>) -> Self::Result {
        let Lookup(user_id) = lookup;

        self.users.get(&user_id).cloned().ok_or(())
    }
}

impl Handler<LookupMany> for Users {
    type Result = Result<(Vec<UserAddress>, Vec<UserId>), ()>;

    fn handle(&mut self, lookup_many: LookupMany, _: &mut Context<Self>) -> Self::Result {
        let LookupMany(user_ids) = lookup_many;

        Ok(user_ids.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut addrs, mut user_ids), user_id| {
                if let Some(addr) = self.users.get(&user_id).cloned() {
                    addrs.push(addr);
                } else {
                    user_ids.push(user_id);
                }

                (addrs, user_ids)
            },
        ))
    }
}

impl Handler<NewUser> for Users {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, _: NewUser, ctx: &mut Context<Self>) -> Self::Result {
        let posts = self.posts.clone();
        let (user_id, user_address) = self.new_user(posts, ctx.address());

        self.users.insert(user_id, user_address.clone());

        for addr in &self.redundancy {
            addr.send(AnnounceNewUser(user_id, user_address.clone()));
        }

        Ok(user_id)
    }
}

impl Handler<AnnounceNewUser> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceNewUser, _: &mut Context<Self>) -> Self::Result {
        let AnnounceNewUser(user_id, user_address) = msg;

        self.users.insert(user_id, user_address);
    }
}

impl Handler<AnnounceDeleteUser> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceDeleteUser, _: &mut Context<Self>) -> Self::Result {
        let AnnounceDeleteUser(user_id) = msg;

        self.users.remove(&user_id);
    }
}

impl Handler<AnnounceRedundancy> for Users {
    type Result = ();

    fn handle(&mut self, msg: AnnounceRedundancy, ctx: &mut Context<Self>) -> Self::Result {
        let AnnounceRedundancy(users_address) = msg;

        users_address.send(ReplyRedundancy(ctx.address::<SyncAddress<Self>>()));

        self.redundancy.push(users_address);
    }
}

impl Handler<ReplyRedundancy> for Users {
    type Result = ();

    fn handle(&mut self, msg: ReplyRedundancy, _: &mut Context<Self>) -> Self::Result {
        let ReplyRedundancy(users_address) = msg;

        self.redundancy.push(users_address);
    }
}
