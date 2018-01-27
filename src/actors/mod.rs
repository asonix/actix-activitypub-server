// use actix::{Actor, Address};

pub mod dispatch;
pub mod posts;
pub mod user;
pub mod users;

use std::cmp::Ordering;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

pub type PostsId = Id;
pub type UsersId = Id;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PostId(pub PostsId, pub Id, pub Instant);

impl PostId {
    pub fn new(posts_id: PostsId, post_id: Id) -> Self {
        PostId(posts_id, post_id, Instant::now())
    }
}

impl Ord for PostId {
    fn cmp(&self, other: &PostId) -> Ordering {
        self.2.cmp(&other.2)
    }
}

impl PartialOrd for PostId {
    fn partial_cmp(&self, other: &PostId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UserId(pub UsersId, pub Id);

impl UserId {
    pub fn new(users_id: UsersId, user_id: Id) -> Self {
        UserId(users_id, user_id)
    }
}

impl Ord for UserId {
    fn cmp(&self, other: &UserId) -> Ordering {
        if self.0 == other.0 {
            self.1.cmp(&other.1)
        } else {
            self.0.cmp(&other.0)
        }
    }
}

impl PartialOrd for UserId {
    fn partial_cmp(&self, other: &UserId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/* Posts is disjoint
 *
 * Users depends on Posts
 *
 * User is Disjoint (created by Users)
 * Inbox depends on User (created by Users)
 * Outbox depends on User, Users (created by Users)
 *
 * Dispatch depends on Users (created by Outbox)
 */

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::thread;

    use actix::{Actor, Arbiter, SyncAddress, System};
    use actix::msgs::{Execute, SystemExit};
    use futures::{Future, Stream};
    use futures::future;
    use futures::stream::iter_ok;

    use super::{Id, UserId};
    use super::posts::Posts;
    use super::posts::messages::{PeerSize as PostPeerSize, PostSize};
    use super::user::messages::{AcceptFollowRequest, DenyFollowRequest, GetPostIds,
                                GetUserPostIds, NewPostOut, RequestFollow};
    use super::users::{UserAddress, Users};
    use super::users::messages::{Lookup, NewUser, PeerSize as UserPeerSize, UserSize};

    #[test]
    fn users_peering() {
        let system = System::new("test");
        let arbiter = Arbiter::new("test-exec");

        let posts_1: SyncAddress<_> = Posts::new(Id(0)).start();
        let posts_2: SyncAddress<_> = Posts::new(Id(1)).add_peer(posts_1.clone()).start();
        let posts_3: SyncAddress<_> = Posts::new(Id(2)).add_peer(posts_2.clone()).start();

        let users_1: SyncAddress<_> = Users::new(Id(0), posts_1.clone()).start();
        let users_2: SyncAddress<_> = Users::new(Id(1), posts_2.clone())
            .add_peer(users_1.clone())
            .start();
        let users_3: SyncAddress<_> = Users::new(Id(2), posts_2.clone())
            .add_peer(users_2.clone())
            .start();

        let fut_1 = posts_1
            .call_fut(PostPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "posts_1 has {} peers, should have {}", peer_size, 3));

        let fut_2 = posts_2
            .call_fut(PostPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "posts_2 has {} peers, should have {}", peer_size, 3));

        let fut_3 = posts_3
            .call_fut(PostPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "posts_3 has {} peers, should have {}", peer_size, 3));

        let fut_4 = users_1
            .call_fut(UserPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "users_1 has {} peers, should have {}", peer_size, 3));

        let fut_5 = users_2
            .call_fut(UserPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "users_2 has {} peers, should have {}", peer_size, 3));

        let fut_6 = users_3
            .call_fut(UserPeerSize)
            .map_err(|_| ())
            .and_then(|res| res)
            .map(|peer_size| assert_eq!(peer_size, 3, "users_3 has {} peers, should have {}", peer_size, 3));

        let fut = fut_1
            .and_then(|_| fut_2)
            .and_then(|_| fut_3)
            .and_then(|_| fut_4)
            .and_then(|_| fut_5)
            .and_then(|_| fut_6);

        thread::sleep(Duration::from_secs(1));

        Arbiter::handle().spawn(
            arbiter
                .call_fut(Execute::new(|| Ok(Arbiter::name())))
                .map_err(|_| ())
                .and_then(|_: Result<_, ()>| fut)
                .map(|_| Arbiter::system().send(SystemExit(0)))
                .map_err(|_| panic!("Future error case")),
                );

        system.run();
    }

    #[test]
    fn test_new_users() {
        with_users(|_, _| future::result(Ok(())))
    }

    #[test]
    fn test_no_follow_and_no_post_propagation() {
        with_users(|ids_vec, addrs_vec| {
            // User 0 requests to follow User 1
            addrs_vec[0].outbox().send(RequestFollow(ids_vec[1]));

            // user 1 accepts user 0's follow request
            addrs_vec[1].outbox().send(DenyFollowRequest(ids_vec[0]));

            // user 1 makes post
            addrs_vec[1].outbox().send(NewPostOut);

            // user 1 should own a post
            let fut = addrs_vec[1]
                .user()
                .call_fut(GetUserPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 0 should not have a post in inbox
            let fut2 = addrs_vec[0]
                .user()
                .call_fut(GetPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            // user 2 should not have a post in inbox
            let fut3 = addrs_vec[2]
                .user()
                .call_fut(GetPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            fut.and_then(|_| fut2).and_then(|_| fut3)
        })
    }

    #[test]
    fn test_follow_and_post_propagation() {
        with_users(|ids_vec, addrs_vec| {
            // User 0 requests to follow User 1
            addrs_vec[0].outbox().send(RequestFollow(ids_vec[1]));

            // User 2 requests to follow User 1
            addrs_vec[2].outbox().send(RequestFollow(ids_vec[1]));

            // user 1 accepts user 0's follow request
            addrs_vec[1].outbox().send(AcceptFollowRequest(ids_vec[0]));

            // user 1 accepts user 2's follow request
            addrs_vec[1].outbox().send(AcceptFollowRequest(ids_vec[2]));

            // user 1 makes post
            addrs_vec[1].outbox().send(NewPostOut);

            // user 1 owns post
            let fut = addrs_vec[1]
                .user()
                .call_fut(GetUserPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 0 should have a post in inbox
            let fut2 = addrs_vec[0]
                .user()
                .call_fut(GetPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 2 should have a post in inbox
            let fut3 = addrs_vec[2]
                .user()
                .call_fut(GetPostIds)
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            fut.and_then(|_| fut2).and_then(|_| fut3)
        })
    }

    fn with_users<F, G>(f: F)
    where
        F: FnOnce(Vec<UserId>, Vec<UserAddress>) -> G + 'static,
        G: Future<Item = (), Error = ()>,
    {
        let system = System::new("test");
        let arbiter = Arbiter::new("test-exec");

        let posts: SyncAddress<_> = Posts::new(Id(0)).start();
        let users: SyncAddress<_> = Users::new(Id(0), posts).start();

        let users2 = users.clone();

        let user_addrs_fut = iter_ok(0..3)
            .and_then(move |_| users.call_fut(NewUser))
            .map_err(|_| ())
            .and_then(|res| res)
            .and_then(move |user_id| {
                users2
                    .call_fut(Lookup(user_id))
                    .map(move |res| res.map(|user_addr| (user_id, user_addr)))
                    .map_err(|_| ())
            })
            .and_then(|res| res)
            .fold(Vec::new(), |mut acc, (user_id, user_addr)| {
                acc.push((user_id, user_addr));
                Ok(acc)
            })
            .and_then(|users| {
                let (ids, addrs) = users.into_iter().unzip();
                f(ids, addrs);
                Ok(())
            });

        Arbiter::handle().spawn(
            arbiter
                .call_fut(Execute::new(|| Ok(Arbiter::name())))
                .map_err(|_| ())
                .and_then(|_: Result<_, ()>| user_addrs_fut)
                .map(|_| Arbiter::system().send(SystemExit(0)))
                .map_err(|_| panic!("Future error case")),
        );

        system.run();
    }
}
