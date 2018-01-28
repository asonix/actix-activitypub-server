use std::collections::BTreeSet;

use actix::{Actor, Arbiter, Context, Handler, SyncAddress};
use futures::Future;

use super::peered::Peered;
use super::peered::messages::Message;
use super::{PostId, UserId};
use super::user::messages::{FollowRequest, FollowRequestAccepted, FollowRequestDenied, NewPostIn};
use super::users::Users;
use super::users::messages::{Lookup, LookupMany};

pub mod messages;

use self::messages::*;

pub struct Dispatch {
    users: SyncAddress<Peered<Users>>,
}

impl Dispatch {
    pub fn new(users: SyncAddress<Peered<Users>>) -> Self {
        Dispatch { users }
    }
}

impl Actor for Dispatch {
    type Context = Context<Self>;
}

impl Handler<DispatchPost> for Dispatch {
    type Result = ();

    fn handle(&mut self, msg: DispatchPost, _: &mut Context<Self>) -> Self::Result {
        let DispatchPost(post_id, user_id, mentions, recipients) = msg;

        let mut ids_set = BTreeSet::new();
        ids_set.extend(recipients);
        ids_set.extend(mentions.clone());

        let fut = self.users
            .call_fut(Message::new(LookupMany(ids_set.into_iter().collect())))
            .map_err(|e| error!("Error: {}", e))
            .and_then(|result| result)
            .and_then(move |(addrs, _)| {
                for addr in addrs {
                    addr.inbox()
                        .send(NewPostIn(post_id, user_id, mentions.clone()));
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
            .call_fut(Message::new(Lookup(target_user)))
            .map_err(|e| error!("Error: {}", e))
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
            .call_fut(Message::new(Lookup(target_user)))
            .map_err(|e| error!("Error: {}", e))
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
            .call_fut(Message::new(Lookup(target_user)))
            .map_err(|e| error!("Error: {}", e))
            .and_then(|result| result)
            .map(move |addr| addr.inbox().send(FollowRequestDenied(denying_user)));

        Arbiter::handle().spawn(fut);
    }
}
