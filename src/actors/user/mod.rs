use std::collections::BTreeSet;

use actix::{Actor, Context, Handler};

use super::{PostId, UserId};

pub mod inbox;
pub mod messages;
pub mod outbox;

use self::messages::*;

pub struct User {
    user_id: UserId,
    posts: BTreeSet<PostId>,
    my_posts: BTreeSet<PostId>,
    followers: BTreeSet<UserId>,
    following: BTreeSet<UserId>,
    follow_requests: BTreeSet<UserId>,
    pending_follows: BTreeSet<UserId>,
    blocklist: BTreeSet<UserId>,
}

impl User {
    pub fn new(user_id: UserId) -> Self {
        User {
            user_id: user_id,
            posts: BTreeSet::new(),
            my_posts: BTreeSet::new(),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            follow_requests: BTreeSet::new(),
            pending_follows: BTreeSet::new(),
            blocklist: BTreeSet::new(),
        }
    }

    pub fn get_10_user_post_ids(&self) -> Vec<PostId> {
        self.my_posts.iter().rev().take(10).cloned().collect()
    }

    pub fn get_10_post_ids(&self) -> Vec<PostId> {
        let p1 = self.posts.iter().rev().peekable();
        let p2 = self.my_posts.iter().rev().peekable();

        // Basically merge-sort
        let (_, _, post_ids) =
            (0..10).fold((p1, p2, Vec::new()), |(mut p1, mut p2, mut vec), _| {
                let use_p2 = {
                    let joined = p1.peek().and_then(|v1| p2.peek().map(|v2| (v1, v2)));

                    if let Some((v1, v2)) = joined {
                        Some(v1 < v2)
                    } else {
                        None
                    }
                };

                let use_p2 = if let Some(use_p2) = use_p2 {
                    use_p2
                } else {
                    p2.peek().is_some()
                };

                let post_id = if use_p2 {
                    p2.next().cloned()
                } else {
                    p1.next().cloned()
                };

                if let Some(post_id) = post_id {
                    vec.push(post_id)
                }

                (p1, p2, vec)
            });

        post_ids
    }

    fn new_post(&mut self, post_id: PostId, user_id: UserId, mentions: &BTreeSet<UserId>) {
        debug!(
            "user {:?} is storing new post {:?} from user {:?}",
            self.user_id, post_id, user_id
        );

        if user_id == self.user_id {
            self.my_posts.insert(post_id);
        } else if self.following.contains(&user_id)
            || (mentions.contains(&self.user_id) && !self.blocklist.contains(&user_id))
        {
            self.posts.insert(post_id);
        } else {
            error!("Should not have recieved post from user {:?}", user_id);
        }
    }

    fn followers(&self) -> Vec<UserId> {
        debug!("followers requested for user {:?}", self.user_id);
        self.followers.iter().cloned().collect()
    }

    fn follow_request(&mut self, user_id: UserId) {
        debug!(
            "user {:?} received follow request from user {:?}",
            self.user_id, user_id
        );
        self.follow_requests.insert(user_id);
    }

    fn accept_follow_request(&mut self, user_id: UserId) -> Option<UserId> {
        self.answer_follow_request(user_id).map(|user_id| {
            self.followers.insert(user_id);
            self.user_id
        })
    }

    fn deny_follow_request(&mut self, user_id: UserId) -> Option<UserId> {
        self.answer_follow_request(user_id).map(|_| self.user_id)
    }

    fn answer_follow_request(&mut self, user_id: UserId) -> Option<UserId> {
        debug!(
            "user {:?} is answering follow request from user {:?}",
            self.user_id, user_id
        );
        if self.follow_requests.remove(&user_id) {
            Some(user_id)
        } else {
            None
        }
    }

    fn request_follow(&mut self, user_id: UserId) {
        self.pending_follows.insert(user_id);
    }

    fn follow_request_accepted(&mut self, user_id: UserId) {
        self.pending_follows.remove(&user_id);
        self.following.insert(user_id);
    }

    fn follow_request_denied(&mut self, user_id: UserId) {
        self.pending_follows.remove(&user_id);
    }
}

impl Actor for User {
    type Context = Context<Self>;
}

impl Handler<NewPostIn> for User {
    type Result = ();

    fn handle(&mut self, msg: NewPostIn, _: &mut Context<Self>) -> Self::Result {
        self.new_post(msg.0, msg.1, &msg.2);
    }
}

impl Handler<GetPostIds> for User {
    type Result = Result<Vec<PostId>, ()>;

    fn handle(&mut self, _: GetPostIds, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_10_post_ids())
    }
}

impl Handler<GetUserPostIds> for User {
    type Result = Result<Vec<PostId>, ()>;

    fn handle(&mut self, _: GetUserPostIds, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_10_user_post_ids())
    }
}

impl Handler<GetFollowers> for User {
    type Result = Result<Vec<UserId>, ()>;

    fn handle(&mut self, _: GetFollowers, _: &mut Context<Self>) -> Self::Result {
        Ok(self.followers())
    }
}

impl Handler<FollowRequest> for User {
    type Result = ();

    fn handle(&mut self, msg: FollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.follow_request(msg.0);
    }
}

impl Handler<AcceptFollowRequest> for User {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: AcceptFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.accept_follow_request(msg.0).ok_or(())
    }
}

impl Handler<DenyFollowRequest> for User {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: DenyFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.deny_follow_request(msg.0).ok_or(())
    }
}

impl Handler<RequestFollow> for User {
    type Result = ();

    fn handle(&mut self, msg: RequestFollow, _: &mut Context<Self>) -> Self::Result {
        self.request_follow(msg.0);
    }
}

impl Handler<FollowRequestAccepted> for User {
    type Result = ();

    fn handle(&mut self, msg: FollowRequestAccepted, _: &mut Context<Self>) -> Self::Result {
        self.follow_request_accepted(msg.0);
    }
}

impl Handler<FollowRequestDenied> for User {
    type Result = ();

    fn handle(&mut self, msg: FollowRequestDenied, _: &mut Context<Self>) -> Self::Result {
        self.follow_request_denied(msg.0);
    }
}
