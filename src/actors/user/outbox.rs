use actix::{Actor, ActorFuture, Address, Context, Handler, ResponseFuture, SyncAddress};
use actix::fut::result;

use actors::blocklist::Blocklists;
use actors::blocklist::messages::Block;
use actors::dispatch::Dispatch;
use actors::dispatch::messages::{DispatchAnnounce, DispatchMessage};
use actors::peered::Peered;
use actors::peered::messages::Message;
use actors::posts::Posts;
use actors::posts::messages::{DeletePost, NewPost};
use actors::users::Users;
use super::messages::*;
use super::{User, UserId};

pub struct Outbox {
    user_id: UserId,
    user: Address<User>,
    posts: SyncAddress<Peered<Posts>>,
    dispatch: Address<Dispatch>,
    blocklists: SyncAddress<Peered<Blocklists>>,
}

impl Outbox {
    pub fn new(
        user_id: UserId,
        user: Address<User>,
        posts: SyncAddress<Peered<Posts>>,
        users: SyncAddress<Peered<Users>>,
        blocklists: SyncAddress<Peered<Blocklists>>,
    ) -> Self {
        let dispatch = Dispatch::new(users, blocklists.clone()).start();

        Outbox {
            user_id,
            user,
            posts,
            dispatch,
            blocklists,
        }
    }
}

impl Actor for Outbox {
    type Context = Context<Self>;
}

impl Handler<NewPostOut> for Outbox {
    type Result = ResponseFuture<Self, NewPostOut>;

    fn handle(&mut self, msg: NewPostOut, _: &mut Context<Self>) -> Self::Result {
        let mentions = msg.0;
        let dispatch = self.dispatch.clone();
        let user = self.user.clone();
        let user_2 = user.clone();
        let user_id = self.user_id;
        debug!("user {:?} is creating a new post", user_id);

        let post_message = Message::new(NewPost(user_id, mentions.clone()));

        let a_fut = self.posts
            .call(self, post_message)
            .map_err(|e, _, _| error!("Error: {}", e))
            .and_then(move |post_id_res, outbox, _| {
                user_2
                    .call(outbox, GetFollowers)
                    .map_err(|e, _, _| error!("Error: {}", e))
                    .and_then(move |followers_res, _, _| result(Ok((post_id_res, followers_res))))
            })
            .and_then(|(post_id_res, followers_res), _, _| {
                let res = post_id_res
                    .and_then(|post_id| followers_res.map(|followers| (post_id, followers)));

                result(res)
            })
            .map(move |(post_id, followers), _, _| {
                debug!("Dispatching {:?} to followers: {:?}", post_id, followers);
                user.send(NewPostIn(post_id, user_id, mentions.clone()));

                dispatch.send(DispatchAnnounce(
                    NewPostIn(post_id, user_id, mentions.clone()),
                    user_id,
                    followers,
                ));

                post_id
            });

        Box::new(a_fut)
    }
}

impl Handler<DeletePost> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: DeletePost, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);
        self.posts.send(Message::new(msg));
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

        self.dispatch.send(DispatchMessage(
            FollowRequest(self.user_id),
            self.user_id,
            msg.0,
        ));
    }
}

impl Handler<AcceptFollowRequest> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: AcceptFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        self.dispatch.send(DispatchMessage(
            FollowRequestAccepted(self.user_id),
            self.user_id,
            msg.0,
        ));
    }
}

impl Handler<DenyFollowRequest> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: DenyFollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.user.send(msg);

        self.dispatch.send(DispatchMessage(
            FollowRequestDenied(self.user_id),
            self.user_id,
            msg.0,
        ));
    }
}

impl Handler<BlockUser> for Outbox {
    type Result = ();

    fn handle(&mut self, msg: BlockUser, _: &mut Context<Self>) -> Self::Result {
        self.dispatch
            .send(DispatchMessage(Blocked(self.user_id), self.user_id, msg.0));

        self.blocklists
            .send(Message::new(Block(self.user_id, msg.0)));
    }
}
