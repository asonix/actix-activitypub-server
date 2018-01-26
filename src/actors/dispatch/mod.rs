use actix::{Actor, Arbiter, Context, Handler, SyncAddress};
use futures::Future;

use super::{PostId, UserId};
use super::user::messages::{FollowRequest, FollowRequestAccepted, FollowRequestDenied, NewPostIn};
use super::users::Users;
use super::users::messages::{Lookup, LookupMany};

pub mod messages;

use self::messages::*;

pub struct Dispatch {
    users: SyncAddress<Users>,
}

impl Dispatch {
    pub fn new(users: SyncAddress<Users>) -> Self {
        Dispatch { users }
    }
}

impl Actor for Dispatch {
    type Context = Context<Self>;
}

impl Handler<DispatchPost> for Dispatch {
    type Result = ();

    fn handle(&mut self, msg: DispatchPost, _: &mut Context<Self>) -> Self::Result {
        let DispatchPost(post_id, user_ids) = msg;

        let fut = self.users
            .call_fut(LookupMany(user_ids))
            .map_err(|e| println!("Error: {}", e))
            .and_then(|result| result)
            .and_then(move |(addrs, _)| {
                for addr in addrs {
                    addr.inbox().send(NewPostIn(post_id));
                }

                Ok(())
            });

        Arbiter::handle().spawn(fut);
    }
}

impl Handler<DispatchFollowRequest> for Dispatch {
    type Result = ();

    fn handle(&mut self, msg: DispatchFollowRequest, _: &mut Context<Self>) -> Self::Result {
        let DispatchFollowRequest {
            requesting_user,
            target_user,
        } = msg;

        let fut = self.users
            .call_fut(Lookup(target_user))
            .map_err(|e| println!("Error: {}", e))
            .and_then(|result| result)
            .map(move |addr| {
                addr.inbox().send(FollowRequest(requesting_user));
            });

        Arbiter::handle().spawn(fut);
    }
}

impl Handler<DispatchAcceptFollowRequest> for Dispatch {
    type Result = ();

    fn handle(&mut self, msg: DispatchAcceptFollowRequest, _: &mut Context<Self>) -> Self::Result {
        let DispatchAcceptFollowRequest {
            accepting_user,
            target_user,
        } = msg;

        let fut = self.users
            .call_fut(Lookup(target_user))
            .map_err(|e| println!("Error: {}", e))
            .and_then(|result| result)
            .map(move |addr| addr.inbox().send(FollowRequestAccepted(accepting_user)));

        Arbiter::handle().spawn(fut);
    }
}

impl Handler<DispatchDenyFollowRequest> for Dispatch {
    type Result = ();

    fn handle(&mut self, msg: DispatchDenyFollowRequest, _: &mut Context<Self>) -> Self::Result {
        let DispatchDenyFollowRequest {
            denying_user,
            target_user,
        } = msg;

        let fut = self.users
            .call_fut(Lookup(target_user))
            .map_err(|e| println!("Error: {}", e))
            .and_then(|result| result)
            .map(move |addr| addr.inbox().send(FollowRequestDenied(denying_user)));

        Arbiter::handle().spawn(fut);
    }
}
