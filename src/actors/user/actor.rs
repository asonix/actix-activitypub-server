use std::collections::BTreeSet;

use actix::{Actor, Context, Handler};

use super::{PostId, User, UserId};
use super::messages::*;

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
    type Result = Result<BTreeSet<PostId>, ()>;

    fn handle(&mut self, _: GetPostIds, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_10_post_ids())
    }
}

impl Handler<GetUserPostIds> for User {
    type Result = Result<BTreeSet<PostId>, ()>;

    fn handle(&mut self, _: GetUserPostIds, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_10_user_post_ids())
    }
}

impl Handler<GetFollowers> for User {
    type Result = Result<BTreeSet<UserId>, ()>;

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
