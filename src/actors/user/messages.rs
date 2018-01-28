use std::collections::BTreeSet;

use actix::{ResponseType, SyncAddress};

use super::{PostId, UserId};
use actors::peered::Peered;
use actors::posts::Posts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewPostIn(pub PostId, pub UserId, pub BTreeSet<UserId>);

impl ResponseType for NewPostIn {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewPostOut(pub BTreeSet<UserId>);

impl ResponseType for NewPostOut {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetPostIds;

impl ResponseType for GetPostIds {
    type Item = BTreeSet<PostId>;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetUserPostIds;

impl ResponseType for GetUserPostIds {
    type Item = BTreeSet<PostId>;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetPosts;

impl ResponseType for GetPosts {
    type Item = SyncAddress<Peered<Posts>>;
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GetFollowers;

impl ResponseType for GetFollowers {
    type Item = BTreeSet<UserId>;
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
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DenyFollowRequest(pub UserId);

impl ResponseType for DenyFollowRequest {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestFollow(pub UserId);

impl ResponseType for RequestFollow {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockUser(pub UserId);

impl ResponseType for BlockUser {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blocked(pub UserId);

impl ResponseType for Blocked {
    type Item = ();
    type Error = ();
}
