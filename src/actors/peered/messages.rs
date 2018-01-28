use std::marker::PhantomData;

use actix::{ResponseType, SyncAddress};

use super::{HandleMessage, Peered, PeeredInner};

pub struct AnnouncePeer<T>(pub SyncAddress<Peered<T>>)
where
    T: PeeredInner + 'static;

impl<T> ResponseType for AnnouncePeer<T>
where
    T: PeeredInner + 'static,
{
    type Item = ();
    type Error = ();
}

pub struct RequestPeers<T>(pub SyncAddress<Peered<T>>)
where
    T: PeeredInner + 'static;

impl<T> ResponseType for RequestPeers<T>
where
    T: PeeredInner + 'static,
{
    type Item = ();
    type Error = ();
}

pub struct ReplyPeers<T>(pub Vec<SyncAddress<Peered<T>>>)
where
    T: PeeredInner + 'static;

impl<T> ResponseType for ReplyPeers<T>
where
    T: PeeredInner + 'static,
{
    type Item = ();
    type Error = ();
}

pub struct RequestBackfill<T>(pub SyncAddress<Peered<T>>, pub T::Request)
where
    T: PeeredInner + 'static;

impl<T> ResponseType for RequestBackfill<T>
where
    T: PeeredInner + 'static,
{
    type Item = ();
    type Error = ();
}

pub struct ReplyBackfill<T>(pub T::Backfill)
where
    T: PeeredInner + 'static;

impl<T> ResponseType for ReplyBackfill<T>
where
    T: PeeredInner + 'static,
{
    type Item = ();
    type Error = ();
}

pub struct PeerSize;

impl ResponseType for PeerSize {
    type Item = usize;
    type Error = ();
}

pub struct Message<T, M>(pub M, pub PhantomData<T>)
where
    T: HandleMessage<M> + PeeredInner + 'static;

impl<T, M> Message<T, M>
where
    T: HandleMessage<M> + PeeredInner + 'static,
{
    pub fn new(message: M) -> Self {
        Message(message, PhantomData)
    }
}

impl<T, M> ResponseType for Message<T, M>
where
    T: HandleMessage<M> + PeeredInner + 'static,
{
    type Item = T::Item;
    type Error = T::Error;
}

pub struct Announce<B>(pub B)
where
    B: Clone + Send;

impl<B> Announce<B>
where
    B: Clone + Send,
{
    pub fn new(broadcast: B) -> Self {
        Announce(broadcast)
    }
}

impl<B> ResponseType for Announce<B>
where
    B: Clone + Send,
{
    type Item = ();
    type Error = ();
}
