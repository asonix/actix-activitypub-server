use actix::{ResponseType, SyncAddress};

use super::{PostId, UserId};
use actors::posts::Posts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NewPostIn(pub PostId, pub UserId);

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

pub struct GetPostIds;

impl ResponseType for GetPostIds {
    type Item = Vec<PostId>;
    type Error = ();
}

pub struct GetUserPostIds;

impl ResponseType for GetUserPostIds {
    type Item = Vec<PostId>;
    type Error = ();
}

pub struct GetPosts;

impl ResponseType for GetPosts {
    type Item = SyncAddress<Posts>;
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
