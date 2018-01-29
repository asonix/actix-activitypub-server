use actix::{Actor, Address, Arbiter, Context, Handler, SyncAddress};
use futures::Future;

use actors::peered::Peered;
use actors::peered::messages::Message;
use actors::posts::messages::DeletePost;
use actors::users::Users;
use actors::users::messages::Lookup;
use super::messages::*;
use super::User;

pub struct Inbox {
    user: Address<User>,
    users: SyncAddress<Peered<Users>>,
}

impl Inbox {
    pub fn new(user: Address<User>, users: SyncAddress<Peered<Users>>) -> Self {
        Inbox { user, users }
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
        let user = self.user.clone();

        let fut = self.users
            .call_fut(Message::new(Lookup(msg.0)))
            .map_err(|e| error!("Error: {}", e))
            .and_then(|res| res)
            .and_then(|addr| {
                addr.user()
                    .call_fut(GetUserPostIds(0))
                    .map_err(|e| error!("Error: {}", e))
                    .and_then(|res| res)
            })
            .map(move |post_ids| {
                for post_id in post_ids {
                    user.send(DeletePost(post_id));
                }
            });

        Arbiter::handle().spawn(fut);
    }
}
