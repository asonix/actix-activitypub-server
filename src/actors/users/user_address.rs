use actix::{Actor, Address, SyncAddress};

use super::{Inbox, Outbox, Posts, User, UserId, Users};

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
