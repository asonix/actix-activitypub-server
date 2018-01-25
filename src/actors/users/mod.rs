use std::collections::HashMap;

use actix::{Actor, Address, Context, Handler};

use super::{Id, UserId};
use super::dispatch::Dispatch;
use super::posts::Posts;
use super::user::User;
use super::user::inbox::Inbox;
use super::user::outbox::Outbox;

pub mod messages;

use self::messages::*;

#[derive(Clone)]
pub struct UserAddress {
    user: Address<User>,
    inbox: Address<Inbox>,
    outbox: Address<Outbox>,
}

impl UserAddress {
    pub fn new(user_id: UserId, posts: Address<Posts>, dispatch: Address<Dispatch>) -> Self {
        let user: Address<_> = User::new(user_id).start();

        let inbox = Inbox::new(user.clone()).start();
        let outbox = Outbox::new(user_id, user.clone(), posts, dispatch).start();

        UserAddress {
            user,
            inbox,
            outbox,
        }
    }

    pub fn user(&self) -> &Address<User> {
        &self.user
    }

    pub fn inbox(&self) -> &Address<Inbox> {
        &self.inbox
    }

    pub fn outbox(&self) -> &Address<Outbox> {
        &self.outbox
    }
}

pub struct Users {
    current_id: u64,
    users: HashMap<UserId, UserAddress>,
}

impl Users {
    pub fn new() -> Self {
        Users::default()
    }

    pub fn new_user(&mut self, posts: Address<Posts>, dispatch: Address<Dispatch>) -> UserId {
        let user_id = self.gen_next_id();
        let user_address = UserAddress::new(user_id, posts, dispatch);

        self.users.insert(user_id, user_address);

        user_id
    }

    fn gen_next_id(&mut self) -> UserId {
        let id = self.current_id;
        self.current_id += 1;
        Id(id)
    }
}

impl Default for Users {
    fn default() -> Self {
        Users {
            current_id: 0,
            users: HashMap::new(),
        }
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
