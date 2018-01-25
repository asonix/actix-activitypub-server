use actix::{Actor, Address, Arbiter, Context, Handler};
use futures::Future;

use actors::posts::Posts;
use actors::posts::messages::NewPost;
use actors::dispatch::Dispatch;
use actors::dispatch::messages::{DispatchAcceptFollowRequest, DispatchDenyFollowRequest,
                                 DispatchFollowRequest, DispatchPost};
use super::messages::*;
use super::{User, UserId};

pub struct Outbox {
    user_id: UserId,
    user: Address<User>,
    posts: Address<Posts>,
    dispatch: Address<Dispatch>,
}

impl Outbox {
    pub fn new(
        user_id: UserId,
        user: Address<User>,
        posts: Address<Posts>,
        dispatch: Address<Dispatch>,
    ) -> Self {
        Outbox {
            user_id,
            user,
            posts,
            dispatch,
        }
    }
}

impl Actor for Outbox {
    type Context = Context<Self>;
}

impl Handler<NewPostOut> for Outbox {
    type Result = ();

    fn handle(&mut self, _: NewPostOut, _: &mut Context<Self>) -> Self::Result {
        let dispatch = self.dispatch.clone();
        let user = self.user.clone();

        let fut = self.posts
            .call_fut(NewPost)
            .join(self.user.call_fut(GetFollowers))
            .map_err(|_| ())
            .and_then(move |(post_result, followers_result)| {
                let res = post_result.and_then(|pid| followers_result.map(|f| (pid, f)));

                if let Ok((post_id, recipients)) = res {
                    user.send(NewPostIn(post_id));

                    dispatch.send(DispatchPost(post_id, recipients));
                }

                Ok(())
            });

        Arbiter::handle().spawn(fut);
    }
}

impl Handler<RequestFollow> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: RequestFollow, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        let RequestFollow(user_id) = msg;

        self.dispatch
            .send(DispatchFollowRequest::new(self.user_id, user_id));
    }
}

impl Handler<AcceptFollowRequest> for Outbox {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: AcceptFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        let AcceptFollowRequest(user_id) = msg;

        self.dispatch
            .send(DispatchAcceptFollowRequest::new(self.user_id, user_id));

        Ok(self.user_id)
    }
}

impl Handler<DenyFollowRequest> for Outbox {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: DenyFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        let DenyFollowRequest(user_id) = msg;

        self.dispatch
            .send(DispatchDenyFollowRequest::new(self.user_id, user_id));

        Ok(self.user_id)
    }
}
