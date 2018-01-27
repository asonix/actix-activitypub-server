use actix::{Actor, AsyncContext, Context, Handler, SyncAddress};

pub mod messages;

use self::messages::*;

pub trait PeeredInner {
    /// The type of data that is used to backfill the type
    type Backfill;
    /// The type of data used to request part of the backfill data
    type Request;

    /// This method retrieves backfill data
    fn backfill(&self, req: Self::Request) -> Self::Backfill;

    /// This method creates the initial backfill request
    fn backfill_init(&self) -> Self::Request;

    /// This method handles incoming backfill data, returning whether or not more should be
    /// requested.
    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request>;
}

pub trait HandleMessage {
    type Message;
    type Broadcast;
    type Item;
    type Error;

    type Result = (Result<Self::Item, Self::Error>, Option<Self::Broadcast>);

    /// Handle an incomming message, returning a response and an optional broacast message
    fn handle_message(message: Self::Message) -> Self::Result;
}

pub trait HandleAnnounce {
    type Broadcast;
    type Item;
    type Error;

    type Result = Result<Self::Item, Self::Error>;
    /// Handle an incomming broadcast message, returning a response
    fn handle_announce(broadcast: Self::Broadcast) -> Self::Result;
}

pub struct Peered<T> {
    inner: T,
    peers: Vec<SyncAddress<Peered<T>>>,
}

impl<T> Peered<T> {
    fn new(inner: T) -> Self {
        Peered {
            inner: inner,
            peers: Vec::new(),
        }
    }

    fn add_peer(mut self, peer: SyncAddress<Peered<T>>) -> Self {
        self.peers.push(peer);
        self
    }

    fn peer_size(&self) -> usize {
        self.peers.len()
    }
}

impl<T> Actor for Peered<T> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if !self.peers.is_empty() {
            self.peers[0].send(RequestPeers(ctx.address()));
        }
    }
}

impl<T> Actor for Peered<T>
where
    T: PeeredInner,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if !self.peers.is_empty() {
            self.peers[0].send(RequestPeers(ctx.address()));
            self.peers[0].send(RequestBackfill(ctx.address(), self.inner.backfill_init()));
        }
    }
}

impl<T> Handler<AnnouncePeer<T>> for Peered<T> {
    type Result = ();

    fn handle(&mut self, msg: AnnouncePeer<T>, _: &mut Context<Self>) -> Self::Result {
        self.peers.push(msg.0);
    }
}

impl<T> Handler<RequestPeers<T>> for Peered<T> {
    type Result = ();

    fn handle(&mut self, msg: RequestPeers<T>, _: &mut Context<Self>) -> Self::Result {
        msg.0.send(ReplyPeers(self.peers.clone()));

        self.peers.push(msg.0);
    }
}

impl<T> Handler<ReplyPeers<T>> for Peered<T> {
    type Result = ();

    fn handle(&mut self, msg: ReplyPeers<T>, ctx: &mut Context<Self>) -> Self::Result {
        for peer in &msg.0 {
            peer.send(AnnouncePeer(ctx.address()));
        }

        self.peers.extend(msg.0);
    }
}

impl<T> Handler<RequestBackfill<T>> for Peered<T>
where
    T: PeeredInner,
{
    type Result = ();

    fn handle(&mut self, msg: RequestBackfill<T>, _: &mut Context<Self>) -> Self::Result {
        let backfill = self.inner.backfill(msg.1);

        msg.0.send(ReplyBackfill(backfill));
    }
}

impl<T, B> Handler<ReplyBackfill<B>> for Peered<T>
where
    T: PeeredInner,
    B: PeeredInner
{
    type Result = ();

    fn handle(&mut self, msg: ReplyBackfill<B>, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(request) = self.inner.handle_backfill(msg.0) {
            self.peers[0].send(RequestBackfill(ctx.address(), request));
        }
    }
}

impl<T> Handler<PeerSize> for Peered<T> {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: PeerSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.peer_size())
    }
}

impl<T> Handler<Message<T>> for Peered<T>
where
    T: HandleMessage,
{
    type Result = Result<T::Item, T::Error>;

    fn handle(&mut self, msg: Message<T>, _: &mut Context<Self>) -> Self::Result {
        let (response, broadcast) = self.inner.handle_message(msg.0);

        if let Some(broadcast) = broadcast {
            for peer in &self.peers {
                peer.send(Announce(broadcast.clone(), (), ()));
            }
        }

        response
    }
}

impl<T> Handler<Announce<T>> for Peered<T>
where
    T: HandleAnnounce,
{
    type Result = Result<T::Item, T::Error>;

    fn handle(&mut self, msg: Announce<T>, _: &mut Context<Self>) -> Self::Result {
        self.inner.handle_announce(msg.0)
    }
}
