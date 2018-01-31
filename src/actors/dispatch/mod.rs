use actix::{Actor, ActorFuture, Context, Handler, ResponseType, ResponseFuture, SyncAddress};
use actix::fut::result;

use super::blocklist::Blocklists;
use super::blocklist::messages::{CanSpeak, GetBlockedBy, GetBlocklist};
use super::peered::Peered;
use super::peered::messages::Message;
use super::user::inbox::Inbox;
use super::UserId;
use super::users::Users;
use super::users::messages::{Lookup, LookupMany};

pub mod messages;

use self::messages::*;

pub struct Dispatch {
    users: SyncAddress<Peered<Users>>,
    blocklists: SyncAddress<Peered<Blocklists>>,
}

impl Dispatch {
    pub fn new(
        users: SyncAddress<Peered<Users>>,
        blocklists: SyncAddress<Peered<Blocklists>>,
    ) -> Self {
        Dispatch { users, blocklists }
    }
}

impl Actor for Dispatch {
    type Context = Context<Self>;
}

impl<T> Handler<DispatchMessage<T>> for Dispatch
where
    T: ResponseType<Item = (), Error = ()> + Send + 'static,
    Inbox: Handler<T>,
{
    type Result = ResponseFuture<Self, DispatchMessage<T>>;

    fn handle(&mut self, msg: DispatchMessage<T>, _: &mut Context<Self>) -> Self::Result {
        let DispatchMessage(message, source, target) = msg;

        let blocklists = self.blocklists.clone();

        let fut = self.users
            .call(self, Message::new(Lookup(target)))
            .and_then(move |addr_result, dispatch, _| {
                blocklists
                    .call(dispatch, Message::new(CanSpeak(source, target)))
                    .map(|speak_result, _, _| (addr_result, speak_result))
            })
            .map_err(|e, _, _| error!("Error: {}", e))
            .and_then(|(addr_result, speak_result), _, _| {
                let res = addr_result.and_then(|addr| {
                    speak_result.map(|speak| (addr, speak))
                });

                result(res)
            })
            .map(move |(addr, can_speak), _, _| {
                if can_speak {
                    addr.inbox().send(message);
                }
            });

        Box::new(fut)
    }
}

impl<T> Handler<DispatchAnnounce<T>> for Dispatch
where
    T: ResponseType<Item = (), Error = ()> + Clone + Send + 'static,
    Inbox: Handler<T>,
{
    type Result = ResponseFuture<Self, DispatchAnnounce<T>>;

    fn handle(&mut self, msg: DispatchAnnounce<T>, _: &mut Context<Self>) -> Self::Result {
        let DispatchAnnounce(message, source, recipients) = msg;

        let users = self.users.clone();
        let blocklists = self.blocklists.clone();

        let fut = self.blocklists
            .call(self, Message::new(GetBlocklist(source)))
            .and_then(move |blocklist_res, dispatch, _| {
                blocklists.call(dispatch, Message::new(GetBlockedBy(source)))
                    .map(|blocked_by_res, _, _| (blocklist_res, blocked_by_res))
            })
            .map_err(|e, _, _| error!("Error: {}", e))
            .and_then(|(blocklist_res, blocked_by_res), _, _| {
                let res = blocklist_res.and_then(|blocklist| {
                    blocked_by_res.map(|blocked_by| (blocklist, blocked_by))
                });

                result(res)
            })
            .map(move |(blocklist, blocked_by), _, _| {
                recipients
                    .difference(&blocklist.union(&blocked_by).cloned().collect())
                    .cloned()
                    .collect()
            })
            .and_then(move |recipients, dispatch, _| {
                users
                    .call(dispatch, Message::new(LookupMany(recipients)))
                    .map_err(|e, _, _| error!("Error: {}", e))
            })
            .and_then(|res, _, _| result(res))
            .map(move |(addrs, _missing_ids), _, _| {
                for addr in addrs {
                    addr.inbox().send(message.clone());
                }
            });

        Box::new(fut)
    }
}
