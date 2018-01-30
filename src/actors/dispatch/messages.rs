use std::collections::BTreeSet;

use actix::ResponseType;

use super::UserId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchMessage<T>(pub T, pub UserId, pub UserId)
where
    T: Send + 'static;

impl<T> ResponseType for DispatchMessage<T>
where
    T: Send + 'static,
{
    type Item = ();
    type Error = ();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchAnnounce<T>(pub T, pub UserId, pub BTreeSet<UserId>)
where
    T: Clone + Send + 'static;

impl<T> ResponseType for DispatchAnnounce<T>
where
    T: Clone + Send + 'static,
{
    type Item = ();
    type Error = ();
}
