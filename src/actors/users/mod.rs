use std::collections::{BTreeMap, BTreeSet};

use actix::SyncAddress;

use super::{Id, UserId, UsersId};
use super::peered::Peered;
use super::posts::Posts;
use super::user::User;
use super::user::inbox::Inbox;
use super::user::outbox::Outbox;
use super::peered::PeeredInner;

mod actor;
pub mod messages;
mod user_address;

pub use self::user_address::UserAddress;

const BACKFILL_CHUNK_SIZE: usize = 100;

pub struct Users {
    users_id: UsersId,
    current_id: u64,
    users: BTreeMap<UserId, UserAddress>,
    posts: SyncAddress<Peered<Posts>>,
}

impl Users {
    pub fn new(users_id: UsersId, posts: SyncAddress<Peered<Posts>>) -> Self {
        Users {
            users_id: users_id,
            current_id: 0,
            users: BTreeMap::new(),
            posts: posts,
        }
    }

    fn gen_next_id(&mut self) -> UserId {
        let id = Id(self.current_id);
        self.current_id += 1;
        UserId(self.users_id, id)
    }

    fn get_user(&self, user_id: UserId) -> Option<UserAddress> {
        self.users.get(&user_id).cloned()
    }

    fn get_users(&self, user_ids: BTreeSet<UserId>) -> (Vec<UserAddress>, Vec<UserId>) {
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

    fn new_user(&mut self, users: SyncAddress<Peered<Users>>) -> (UserId, UserAddress) {
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

impl PeeredInner for Users {
    type Backfill = (usize, Vec<(UserId, UserAddress)>);
    type Request = usize;

    fn backfill(&self, req: Self::Request) -> Self::Backfill {
        let u = self.users
            .iter()
            .skip(req)
            .take(BACKFILL_CHUNK_SIZE)
            .map(|(a, b)| (*a, b.clone()))
            .collect();

        (req, u)
    }

    fn backfill_init(&self) -> Self::Request {
        0
    }

    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request> {
        let ret = if backfill.1.len() == BACKFILL_CHUNK_SIZE {
            Some(backfill.0 + BACKFILL_CHUNK_SIZE)
        } else {
            None
        };

        self.users.extend(backfill.1);

        ret
    }
}
