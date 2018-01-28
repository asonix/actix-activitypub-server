use actix::{Actor, Address, Arbiter, Context, Handler, SyncAddress};
use futures::Future;

use actors::dispatch::Dispatch;
use actors::dispatch::messages::{DispatchAcceptFollowRequest, DispatchDenyFollowRequest,
                                 DispatchFollowRequest, DispatchPost};
use actors::peered::Peered;
use actors::peered::messages::Message;
use actors::posts::Posts;
use actors::posts::messages::NewPost;
use actors::users::Users;
use super::messages::*;
use super::{User, UserId};

pub struct Outbox {
    user_id: UserId,
    user: Address<User>,
    posts: SyncAddress<Peered<Posts>>,
    dispatch: Address<Dispatch>,
}

impl Outbox {
    pub fn new(
        user_id: UserId,
        user: Address<User>,
        posts: SyncAddress<Peered<Posts>>,
        users: SyncAddress<Peered<Users>>,
    ) -> Self {
        let dispatch = Dispatch::new(users).start();

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
        let user_id = self.user_id;
        debug!("user {:?} is creating a new post", user_id);

        let fut = self.posts
            .call_fut(Message::new(NewPost(user_id)))
            .join(self.user.call_fut(GetFollowers))
            .map_err(|_| ())
            .and_then(move |(post_result, followers_result)| {
                let res = post_result.and_then(|pid| followers_result.map(|f| (pid, f)));

                if let Ok((post_id, recipients)) = res {
                    debug!("Dispatching {:?} to recipients: {:?}", post_id, recipients);
                    user.send(NewPostIn(post_id, user_id));

                    dispatch.send(DispatchPost(post_id, user_id, recipients));
                }

                Ok(())
            });

        Arbiter::handle().spawn(fut);
    }
}

impl Handler<RequestFollow> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: RequestFollow, _: &mut Context<Self>) -> Self::Result {
        debug!(
            "user {:?} requesting to follow user {:?}",
            self.user_id, msg.0
        );
        self.user.send(msg);

        self.dispatch
            .send(DispatchFollowRequest::new(self.user_id, msg.0));
    }
}

impl Handler<AcceptFollowRequest> for Outbox {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: AcceptFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        self.dispatch
            .send(DispatchAcceptFollowRequest::new(self.user_id, msg.0));

        Ok(self.user_id)
    }
}

impl Handler<DenyFollowRequest> for Outbox {
    type Result = Result<UserId, ()>;

    fn handle(&mut self, msg: DenyFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        self.dispatch
            .send(DispatchDenyFollowRequest::new(self.user_id, msg.0));

        Ok(self.user_id)
    }
}
