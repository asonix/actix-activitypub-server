use actix::{Actor, AsyncContext, Context, Handler, SyncAddress};

pub mod messages;

use self::messages::*;

pub trait PeeredInner {
    /// The type of data that is used to backfill the type
    type Backfill: Send;
    /// The type of data used to request part of the backfill data
    type Request: Send;

    /// This method retrieves backfill data
    fn backfill(&self, req: Self::Request) -> Self::Backfill;

    /// This method creates the initial backfill request
    fn backfill_init(&self) -> Self::Request;

    /// This method handles incoming backfill data, returning whether or not more should be
    /// requested.
    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request>;
}

pub type HandleMessageType<I, E, B> = (Result<I, E>, Option<B>);

pub trait HandleMessage<M> {
    type Broadcast: Clone + Send + 'static;
    type Item: Send;
    type Error: Send;

    /// Handle an incomming message, returning a response and an optional broacast message
    fn handle_message(
        &mut self,
        message: M,
    ) -> HandleMessageType<Self::Item, Self::Error, Self::Broadcast>;
}

pub trait HandleAnnounce<B> {
    type Item: Send;
    type Error: Send;

    /// Handle an incomming broadcast message, returning a response
    fn handle_announce(&mut self, broadcast: B) -> Result<Self::Item, Self::Error>;
}

pub struct Peered<T>
where
    T: PeeredInner + 'static,
{
    inner: T,
    peers: Vec<SyncAddress<Peered<T>>>,
}

impl<T> Peered<T>
where
    T: PeeredInner + 'static,
{
    pub fn new(inner: T) -> Self {
        Peered {
            inner: inner,
            peers: Vec::new(),
        }
    }

    pub fn add_peer(mut self, peer: SyncAddress<Peered<T>>) -> Self {
        self.peers.push(peer);
        self
    }

    fn peer_size(&self) -> usize {
        self.peers.len()
    }
}

impl<T> Actor for Peered<T>
where
    T: PeeredInner + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if !self.peers.is_empty() {
            self.peers[0].send(RequestPeers(ctx.address()));
            self.peers[0].send(RequestBackfill(ctx.address(), self.inner.backfill_init()));
        }
    }
}

impl<T> Handler<AnnouncePeer<T>> for Peered<T>
where
    T: PeeredInner + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: AnnouncePeer<T>, _: &mut Context<Self>) -> Self::Result {
        self.peers.push(msg.0);
    }
}

impl<T> Handler<RequestPeers<T>> for Peered<T>
where
    T: PeeredInner + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: RequestPeers<T>, _: &mut Context<Self>) -> Self::Result {
        msg.0.send(ReplyPeers(self.peers.clone()));

        self.peers.push(msg.0);
    }
}

impl<T> Handler<ReplyPeers<T>> for Peered<T>
where
    T: PeeredInner + 'static,
{
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
    T: PeeredInner + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: RequestBackfill<T>, _: &mut Context<Self>) -> Self::Result {
        let backfill = self.inner.backfill(msg.1);

        msg.0.send(ReplyBackfill(backfill));
    }
}

impl<T> Handler<ReplyBackfill<T>> for Peered<T>
where
    T: PeeredInner + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: ReplyBackfill<T>, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(request) = self.inner.handle_backfill(msg.0) {
            self.peers[0].send(RequestBackfill(ctx.address(), request));
        }
    }
}

impl<T> Handler<PeerSize> for Peered<T>
where
    T: PeeredInner + 'static,
{
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: PeerSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.peer_size())
    }
}

impl<T, M> Handler<Message<T, M>> for Peered<T>
where
    T: HandleMessage<M>
        + HandleAnnounce<<T as HandleMessage<M>>::Broadcast>
        + PeeredInner
        + 'static,
{
    type Result = Result<<T as HandleMessage<M>>::Item, <T as HandleMessage<M>>::Error>;

    fn handle(&mut self, msg: Message<T, M>, _: &mut Context<Self>) -> Self::Result {
        let (response, broadcast) = self.inner.handle_message(msg.0);

        if let Some(broadcast) = broadcast {
            for peer in &self.peers {
                peer.send(Announce::new(broadcast.clone()));
            }
        }

        response
    }
}

impl<T, B> Handler<Announce<B>> for Peered<T>
where
    T: HandleAnnounce<B> + PeeredInner + 'static,
    B: Clone + Send,
{
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: Announce<B>, _: &mut Context<Self>) -> Self::Result {
        let _ = self.inner.handle_announce(msg.0);
        Ok(())
    }
}

impl<K> HandleAnnounce<()> for K {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, _: ()) -> Result<(), ()> {
        Ok(())
    }
}
