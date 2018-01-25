use actix::ResponseType;

use super::{PostId, UserId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NewPostIn(pub PostId);

impl ResponseType for NewPostIn {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NewPostOut;

impl ResponseType for NewPostOut {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetFollowers;

impl ResponseType for GetFollowers {
    type Item = Vec<UserId>;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FollowRequest(pub UserId);

impl ResponseType for FollowRequest {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceptFollowRequest(pub UserId);

impl ResponseType for AcceptFollowRequest {
    type Item = UserId;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DenyFollowRequest(pub UserId);

impl ResponseType for DenyFollowRequest {
    type Item = UserId;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestFollow(pub UserId);

impl ResponseType for RequestFollow {
    type Item = ();
    type Error = ();
}

pub struct FollowRequestAccepted(pub UserId);

impl ResponseType for FollowRequestAccepted {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FollowRequestDenied(pub UserId);

impl ResponseType for FollowRequestDenied {
    type Item = ();
    type Error = ();
}
