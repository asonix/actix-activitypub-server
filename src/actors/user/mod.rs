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
    followers: BTreeSet<UserId>,
    following: BTreeSet<UserId>,
    follow_requests: BTreeSet<UserId>,
    pending_follows: BTreeSet<UserId>,
}

impl User {
    pub fn new(user_id: UserId) -> Self {
        User {
            user_id: user_id,
            posts: BTreeSet::new(),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            follow_requests: BTreeSet::new(),
            pending_follows: BTreeSet::new(),
        }
    }
}

impl Actor for User {
    type Context = Context<Self>;
}

impl Handler<NewPostIn> for User {
    type Result = ();

    fn handle(&mut self, msg: NewPostIn, _: &mut Context<Self>) -> Self::Result {
        let NewPostIn(post_id) = msg;

        self.posts.insert(post_id);
    }
}

impl Handler<GetFollowers> for User {
    type Result = Result<Vec<UserId>, ()>;

    fn handle(&mut self, _: GetFollowers, _: &mut Context<Self>) -> Self::Result {
        Ok(self.followers.iter().cloned().collect())
    }
}

impl Handler<FollowRequest> for User {
    type Result = ();

    fn handle(&mut self, follow_request: FollowRequest, _: &mut Context<Self>) -> Self::Result {
        let FollowRequest(user_id) = follow_request;

        self.follow_requests.insert(user_id);
    }
}

impl Handler<AcceptFollowRequest> for User {
    type Result = Result<UserId, ()>;

    fn handle(
        &mut self,
        accept_follow_request: AcceptFollowRequest,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let AcceptFollowRequest(user_id) = accept_follow_request;

        self.follow_requests.remove(&user_id);

        Ok(self.user_id)
    }
}

impl Handler<DenyFollowRequest> for User {
    type Result = Result<UserId, ()>;

    fn handle(
        &mut self,
        deny_follow_request: DenyFollowRequest,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let DenyFollowRequest(user_id) = deny_follow_request;

        self.follow_requests.remove(&user_id);

        Ok(self.user_id)
    }
}

impl Handler<RequestFollow> for User {
    type Result = ();

    fn handle(&mut self, request_follow: RequestFollow, _: &mut Context<Self>) -> Self::Result {
        let RequestFollow(user_id) = request_follow;

        self.pending_follows.insert(user_id);
    }
}

impl Handler<FollowRequestAccepted> for User {
    type Result = ();

    fn handle(
        &mut self,
        follow_request_accepted: FollowRequestAccepted,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let FollowRequestAccepted(user_id) = follow_request_accepted;

        self.pending_follows.remove(&user_id);
        self.following.insert(user_id);
    }
}

impl Handler<FollowRequestDenied> for User {
    type Result = ();

    fn handle(
        &mut self,
        follow_request_denied: FollowRequestDenied,
        _: &mut Context<Self>,
    ) -> Self::Result {
        let FollowRequestDenied(user_id) = follow_request_denied;

        self.pending_follows.remove(&user_id);
    }
}
