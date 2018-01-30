use actix::{Actor, Arbiter, Context, Handler, ResponseType, SyncAddress};
use futures::Future;

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
    type Result = ();

    fn handle(&mut self, msg: DispatchMessage<T>, _: &mut Context<Self>) -> Self::Result {
        let DispatchMessage(message, source, target) = msg;

        let fut = self.users
            .call_fut(Message::new(Lookup(target)))
            .join(
                self.blocklists
                    .call_fut(Message::new(CanSpeak(source, target))),
            )
            .map_err(|e| error!("Error: {}", e))
            .and_then(|result| result)
            .map(move |(addr, can_speak)| {
                if can_speak {
                    addr.inbox().send(message);
                }
            });

        Arbiter::handle().spawn(fut);
    }
}

impl<T> Handler<DispatchAnnounce<T>> for Dispatch
where
    T: ResponseType<Item = (), Error = ()> + Clone + Send + 'static,
    Inbox: Handler<T>,
{
    type Result = ();

    fn handle(&mut self, msg: DispatchAnnounce<T>, _: &mut Context<Self>) -> Self::Result {
        let DispatchAnnounce(message, source, recipients) = msg;

        let users = self.users.clone();

        let fut = self.blocklists
            .call_fut(Message::new(GetBlocklist(source)))
            .join(self.blocklists.call_fut(Message::new(GetBlockedBy(source))))
            .map_err(|e| error!("Error: {}", e))
            .and_then(|result| result)
            .map(move |(blocklist, blocked_by)| {
                recipients
                    .difference(&blocklist.union(&blocked_by).cloned().collect())
                    .cloned()
                    .collect()
            })
            .and_then(move |recipients| {
                users
                    .call_fut(Message::new(LookupMany(recipients)))
                    .map_err(|e| error!("Error: {}", e))
            })
            .and_then(|result| result)
            .map(move |(addrs, _missing_ids)| {
                for addr in addrs {
                    addr.inbox().send(message.clone());
                }
            });

        Arbiter::handle().spawn(fut);
    }
}
