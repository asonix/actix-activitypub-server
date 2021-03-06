pub mod blocklist;
pub mod dispatch;
pub mod peered;
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
    use std::collections::BTreeSet;
    use std::time::Duration;

    use actix::{Actor, Arbiter, SyncAddress, System};
    use actix::msgs::{Execute, SystemExit};
    use futures::{Future, Stream};
    use futures::future;
    use futures::stream::iter_ok;
    use tokio_timer::Timer;

    use super::blocklist::Blocklists;
    use super::blocklist::messages::CanSpeak;
    use super::{Id, UserId};
    use super::peered::Peered;
    use super::peered::messages::{Message, PeerSize};
    use super::posts::Posts;
    use super::posts::messages::PostSize;
    use super::user::messages::{AcceptFollowRequest, BlockUser, DenyFollowRequest, GetFollowers,
                                GetPostIds, GetUserPostIds, NewPostOut, RequestFollow};
    use super::users::{UserAddress, Users};
    use super::users::messages::{Lookup, LookupMany, NewUser, UserSize};

    #[test]
    fn peered_users_can_iteract() {
        let system = System::new("test");
        let handle = Arbiter::handle();

        let posts_1: SyncAddress<_> = Peered::new(Posts::new(Id(0))).start();
        let posts_2: SyncAddress<_> = Peered::new(Posts::new(Id(1)))
            .add_peer(posts_1.clone())
            .start();

        let users_1: SyncAddress<_> = Peered::new(Users::new(Id(0), posts_1.clone())).start();
        let users_2: SyncAddress<_> = Peered::new(Users::new(Id(1), posts_2.clone()))
            .add_peer(users_1.clone())
            .start();

        let ping_1 = posts_1.call_fut(Message::new(PostSize));
        let ping_2 = posts_2.call_fut(Message::new(PostSize));
        let ping_3 = users_1.call_fut(Message::new(UserSize));
        let ping_4 = users_2.call_fut(Message::new(UserSize));

        let blocklists: SyncAddress<_> = Peered::new(Blocklists::new()).start();

        let new_u1 = Message::new(NewUser(users_1.clone(), blocklists.clone()));
        let u1 = users_1.call_fut(new_u1).map_err(|_| ()).and_then(|res| res);
        let new_u2 = Message::new(NewUser(users_2.clone(), blocklists.clone()));
        let u2 = users_2.call_fut(new_u2).map_err(|_| ()).and_then(|res| res);

        let duration = Duration::from_millis(200);

        let fut = ping_1
            .join4(ping_2, ping_3, ping_4)
            .map_err(|_| ())
            .and_then(move |_| Timer::default().sleep(duration).map_err(|_| ()))
            .and_then(|_| u1.join(u2).map_err(|_| ()))
            .and_then(move |(uid1, uid2)| {
                let uid_vec = vec![uid1, uid2];

                users_1
                    .call_fut(Message::new(LookupMany(uid_vec.iter().cloned().collect())))
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|(user_addrs, _)| (uid_vec, user_addrs))
            })
            .map(|(ids_vec, addrs_vec)| {
                for addr in &addrs_vec {
                    // ping each address
                    addr.user().send(GetFollowers);
                }

                (ids_vec, addrs_vec)
            })
            .and_then(|(ids_vec, addrs_vec)| {
                assert_eq!(addrs_vec.len(), 2);
                // User 0 requests to follow User 1
                addrs_vec[0]
                    .outbox()
                    .call_fut(RequestFollow(ids_vec[1]))
                    .map_err(|_| ())
                    .map(|_| (ids_vec, addrs_vec))
            })
            .and_then(move |vecs| {
                Timer::default()
                    .sleep(duration)
                    .map_err(|_| ())
                    .map(|_| vecs)
            })
            .and_then(|(ids_vec, addrs_vec)| {
                // user 1 accepts user 0's follow request
                addrs_vec[1]
                    .outbox()
                    .call_fut(AcceptFollowRequest(ids_vec[0]))
                    .map_err(|_| ())
                    .map(|_| (ids_vec, addrs_vec))
            })
            .and_then(move |vecs| {
                Timer::default()
                    .sleep(duration)
                    .map_err(|_| ())
                    .map(|_| vecs)
            })
            .and_then(move |(ids_vec, addrs_vec)| {
                // user 1 makes post
                addrs_vec[1]
                    .outbox()
                    .call_fut(NewPostOut(BTreeSet::new()))
                    .map_err(|_| ())
                    .map(|_| (ids_vec, addrs_vec))
            })
            .and_then(move |vecs| {
                Timer::default()
                    .sleep(duration)
                    .map_err(|_| ())
                    .map(|_| vecs)
            })
            .and_then(move |(_, addrs_vec)| {
                // user 1 should own a post
                let fut = addrs_vec[1]
                    .user()
                    .call_fut(GetUserPostIds(10))
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|post_ids| assert!(!post_ids.is_empty()));

                // user 0 should have a post in inbox
                let fut2 = addrs_vec[0]
                    .user()
                    .call_fut(GetPostIds(10))
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|post_ids| assert!(!post_ids.is_empty()));

                // user 0 should not own a post
                let fut3 = addrs_vec[0]
                    .user()
                    .call_fut(GetUserPostIds(10))
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|post_ids| assert!(post_ids.is_empty()));

                fut.and_then(|_| fut2).and_then(|_| fut3)
            })
            .map(|_| Arbiter::system().send(SystemExit(0)))
            .map_err(|_| panic!("Future error case"));

        handle.spawn(fut);

        system.run();
    }

    #[test]
    fn users_and_posts_peering() {
        let system = System::new("test");
        let arbiter = Arbiter::new("test-exec");
        let handle = Arbiter::handle();

        let posts_1: SyncAddress<_> = Peered::new(Posts::new(Id(0))).start();
        let posts_2: SyncAddress<_> = Peered::new(Posts::new(Id(1)))
            .add_peer(posts_1.clone())
            .start();
        let posts_3: SyncAddress<_> = Peered::new(Posts::new(Id(2)))
            .add_peer(posts_2.clone())
            .start();

        let users_1: SyncAddress<_> = Peered::new(Users::new(Id(0), posts_1.clone())).start();
        let users_2: SyncAddress<_> = Peered::new(Users::new(Id(1), posts_2.clone()))
            .add_peer(users_1.clone())
            .start();
        let users_3: SyncAddress<_> = Peered::new(Users::new(Id(2), posts_2.clone()))
            .add_peer(users_2.clone())
            .start();

        let ping_1 = posts_1.call_fut(Message::new(PostSize));
        let ping_2 = posts_2.call_fut(Message::new(PostSize));
        let ping_3 = posts_3.call_fut(Message::new(PostSize));
        let ping_4 = users_1.call_fut(Message::new(UserSize));
        let ping_5 = users_2.call_fut(Message::new(UserSize));
        let ping_6 = users_3.call_fut(Message::new(UserSize));

        let fut = ping_1
            .and_then(|_| ping_2)
            .and_then(|_| ping_3)
            .and_then(|_| ping_4)
            .and_then(|_| ping_5)
            .and_then(|_| ping_6)
            .map_err(|_| ())
            .and_then(|_| {
                Timer::default()
                    .sleep(Duration::from_secs(1))
                    .map_err(|_| ())
            })
            .and_then(move |_| {
                let fut_1 = posts_1
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                let fut_2 = posts_2
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                let fut_3 = posts_3
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                let fut_4 = users_1
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                let fut_5 = users_2
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                let fut_6 = users_3
                    .call_fut(PeerSize)
                    .map_err(|_| ())
                    .and_then(|res| res)
                    .map(|peer_size| assert_eq!(peer_size, 2));

                fut_1
                    .and_then(|_| fut_2)
                    .and_then(|_| fut_3)
                    .and_then(|_| fut_4)
                    .and_then(|_| fut_5)
                    .and_then(|_| fut_6)
            });

        handle.spawn(
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
        with_users(|_, _, _| future::result(Ok(())))
    }

    #[test]
    fn test_blocked_user_doesnt_receive_post() {
        with_users(|ids_vec, addrs_vec, blocklists| {
            let u0_a = addrs_vec[0].clone();
            let u0_b = addrs_vec[0].clone();
            let uid0 = ids_vec[0];
            let u1_a = addrs_vec[1].clone();
            let u1_b = u1_a.clone();
            let u1_c = u1_a.clone();
            let u1_d = u1_a.clone();
            let uid1 = ids_vec[1];

            let blocklists_clone = blocklists.clone();

            // User 0 requests to follow User 1
            u0_a.outbox()
                .call_fut(RequestFollow(ids_vec[1]))
                .map_err(|_| ())
                .and_then(move |_| {
                    // user 0 and user 1 have no established blocks
                    blocklists
                        .clone()
                        .call_fut(Message::new(CanSpeak(uid0, uid1)))
                        .map_err(|_| ())
                        .and_then(|res| res)
                        .map(|can_speak| assert!(can_speak))
                })
                .map(|_| Timer::default().sleep(Duration::from_millis(100)))
                .and_then(move |_| {
                    // user 1 accepts user 0's follow request
                    u1_a.outbox()
                        .call_fut(AcceptFollowRequest(uid0))
                        .map_err(|_| ())
                })
                .map(|_| Timer::default().sleep(Duration::from_millis(100)))
                .and_then(move |_| {
                    u1_b.outbox()
                        .call_fut(NewPostOut(BTreeSet::new()))
                        .map_err(|_| ())
                })
                .map(|_| Timer::default().sleep(Duration::from_millis(100)))
                .and_then(move |_| {
                    // user 0 should have a post in inbox
                    u0_b.user()
                        .call_fut(GetPostIds(10))
                        .map_err(|_| ())
                        .and_then(|res| res)
                        .map(|post_ids| assert!(!post_ids.is_empty()))
                })
                .map(|_| Timer::default().sleep(Duration::from_millis(100)))
                .and_then(move |_| {
                    // user 1 blocks user 0
                    u1_c.outbox().call_fut(BlockUser(uid0)).map_err(|_| ())
                })
                .map(|_| Timer::default().sleep(Duration::from_millis(100)))
                .and_then(move |_| {
                    // user 1 makes post
                    u1_d.outbox()
                        .call_fut(NewPostOut(BTreeSet::new()))
                        .map_err(|_| ())
                })
                .and_then(move |_| {
                    // user 1 should own two posts
                    let fut = addrs_vec[1]
                        .user()
                        .call_fut(GetUserPostIds(10))
                        .map_err(|_| ())
                        .and_then(|res| res)
                        .map(|post_ids| assert_eq!(post_ids.len(), 2));

                    // user 0 should not have a post in inbox
                    let fut2 = addrs_vec[0]
                        .user()
                        .call_fut(GetPostIds(10))
                        .map_err(|_| ())
                        .and_then(|res| res)
                        .map(|post_ids| assert!(post_ids.is_empty()));

                    // user 1 and user 0 have a block separating them
                    let fut3 = blocklists_clone
                        .call_fut(Message::new(CanSpeak(uid0, uid1)))
                        .map_err(|_| ())
                        .and_then(|res| res)
                        .map(|can_speak| assert!(!can_speak));

                    fut.and_then(|_| fut2).and_then(|_| fut3)
                })
        })
    }

    #[test]
    fn test_no_follow_and_no_post_propagation() {
        with_users(|ids_vec, addrs_vec, _| {
            // User 0 requests to follow User 1
            addrs_vec[0].outbox().send(RequestFollow(ids_vec[1]));

            // user 1 denies user 0's follow request
            addrs_vec[1].outbox().send(DenyFollowRequest(ids_vec[0]));

            // user 1 makes post
            addrs_vec[1].outbox().send(NewPostOut(BTreeSet::new()));

            // user 1 should own a post
            let fut = addrs_vec[1]
                .user()
                .call_fut(GetUserPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 0 should not have a post in inbox
            let fut2 = addrs_vec[0]
                .user()
                .call_fut(GetPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            // user 2 should not have a post in inbox
            let fut3 = addrs_vec[2]
                .user()
                .call_fut(GetPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            fut.and_then(|_| fut2).and_then(|_| fut3)
        })
    }

    #[test]
    fn test_follow_and_post_propagation() {
        with_users(|ids_vec, addrs_vec, _| {
            // User 0 requests to follow User 1
            addrs_vec[0].outbox().send(RequestFollow(ids_vec[1]));

            // User 2 requests to follow User 1
            addrs_vec[2].outbox().send(RequestFollow(ids_vec[1]));

            // user 1 accepts user 0's follow request
            addrs_vec[1].outbox().send(AcceptFollowRequest(ids_vec[0]));

            // user 1 accepts user 2's follow request
            addrs_vec[1].outbox().send(AcceptFollowRequest(ids_vec[2]));

            // user 1 makes post
            addrs_vec[1].outbox().send(NewPostOut(BTreeSet::new()));

            // user 1 owns post
            let fut = addrs_vec[1]
                .user()
                .call_fut(GetUserPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 0 should have a post in inbox
            let fut2 = addrs_vec[0]
                .user()
                .call_fut(GetPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 2 should have a post in inbox
            let fut3 = addrs_vec[2]
                .user()
                .call_fut(GetPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(!post_ids.is_empty()));

            // user 0 should not own a post
            let fut4 = addrs_vec[0]
                .user()
                .call_fut(GetUserPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            // user 2 should not own a post
            let fut5 = addrs_vec[2]
                .user()
                .call_fut(GetUserPostIds(10))
                .map_err(|_| ())
                .and_then(|res| res)
                .map(|post_ids| assert!(post_ids.is_empty()));

            fut.join5(fut2, fut3, fut4, fut5).map(|_| ())
        })
    }

    fn with_users<F, G>(f: F)
    where
        F: FnOnce(Vec<UserId>, Vec<UserAddress>, SyncAddress<Peered<Blocklists>>) -> G + 'static,
        G: Future<Item = (), Error = ()>,
    {
        let system = System::new("test");
        let arbiter = Arbiter::new("test-exec");

        let posts: SyncAddress<_> = Peered::new(Posts::new(Id(0))).start();
        let users: SyncAddress<_> = Peered::new(Users::new(Id(0), posts)).start();
        let blocklists: SyncAddress<_> = Peered::new(Blocklists::new()).start();
        let blocklists_clone = blocklists.clone();

        let users_clone = users.clone();

        let user_addrs_fut = iter_ok(0..3)
            .and_then(move |_| {
                users.call_fut(Message::new(NewUser(users.clone(), blocklists.clone())))
            })
            .map_err(|_| ())
            .and_then(|res| res)
            .and_then(move |user_id| {
                users_clone
                    .call_fut(Message::new(Lookup(user_id)))
                    .map(move |res| res.map(|user_addr| (user_id, user_addr)))
                    .map_err(|_| ())
            })
            .and_then(|res| res)
            .fold(Vec::new(), |mut acc, (user_id, user_addr)| {
                acc.push((user_id, user_addr));
                Ok(acc) as Result<_, ()>
            })
            .and_then(|users| {
                let (ids, addrs) = users.into_iter().unzip();
                f(ids, addrs, blocklists_clone);
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
