use actix::{Actor, Address, Context, Handler};

use super::messages::*;
use super::User;

pub struct Inbox {
    user: Address<User>,
}

impl Inbox {
    pub fn new(user: Address<User>) -> Self {
        Inbox { user }
    }
}

impl Actor for Inbox {
    type Context = Context<Self>;
}

impl Handler<NewPostIn> for Inbox {
    type Result = ();

    fn handle(&mut self, msg: NewPostIn, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
    }
}

impl Handler<FollowRequest> for Inbox {
    type Result = ();

    fn handle(&mut self, msg: FollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
    }
}

impl Handler<FollowRequestAccepted> for Inbox {
    type Result = ();

    fn handle(&mut self, msg: FollowRequestAccepted, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
    }
}

impl Handler<FollowRequestDenied> for Inbox {
    type Result = ();

    fn handle(&mut self, msg: FollowRequestDenied, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
    }
}

impl Handler<Blocked> for Inbox {
    type Result = ();

    fn handle(&mut self, msg: Blocked, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
    }
}
