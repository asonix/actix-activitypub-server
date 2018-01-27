use std::marker::PhantomData;

use actix::{ResponseType, SyncAddress};

use super::{HandleMessage, HandleAnnounce, Peered, PeeredInner};

pub struct AnnouncePeer<T>(pub SyncAddress<Peered<T>>);

impl<T> ResponseType for AnnouncePeer<T> {
    type Item = ();
    type Error = ();
}

pub struct RequestPeers<T>(pub SyncAddress<Peered<T>>);

impl<T> ResponseType for AnnouncePeer<T> {
    type Item = ();
    type Error = ();
}

pub struct ReplyPeers<T>(pub Vec<SyncAddress<Peered<T>>>);

impl<T> ResponseType for ReplyPeers<T> {
    type Item = ();
    type Error = ();
}

pub struct RequestBackfill<T>(pub SyncAddress<Peered<T>>, pub T::Request)
where
    T: PeeredInner;

impl<T> ResponseType for RequestBackfill<T>
where
    T: PeeredInner,
{
    type Item = ();
    type Error = ();
}

pub struct ReplyBackfill<T>(pub T::Request)
where
    T: PeeredInner;

impl<T> ResponseType for ReplyBackfill<T>
where
    T: PeeredInner,
{
    type Item = ();
    type Error = ();
}

pub struct PeerSize;

impl ResponseType for PeerSize {
    type Item = usize;
    type Error = ();
}

pub struct Message<T>(pub T::Message)
where
    T: HandleMessage;

impl<T> ResponseType for Message<T>
where
    T: HandleMessage,
{
    type Item = T::Item;
    type Error = T::Error;
}

pub struct Announce<T>(pub T::Broadcast)
where
    T: HandleAnnounce;

impl<T> ResponseType for Announce<T>
where
    T: HandleAnnounce,
{
    type Item = T::Item;
    type Error = T::Error;
}
