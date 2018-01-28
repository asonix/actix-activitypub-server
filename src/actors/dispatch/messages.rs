use std::collections::BTreeSet;

use actix::ResponseType;

use super::{PostId, UserId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchPost(
    pub PostId,
    pub UserId,
    pub BTreeSet<UserId>,
    pub Vec<UserId>,
);

impl ResponseType for DispatchPost {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchFollowRequest {
    pub requesting_user: UserId,
    pub target_user: UserId,
}

impl DispatchFollowRequest {
    pub fn new(requesting_user: UserId, target_user: UserId) -> Self {
        DispatchFollowRequest {
            requesting_user,
            target_user,
        }
    }
}

impl ResponseType for DispatchFollowRequest {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchAcceptFollowRequest {
    pub accepting_user: UserId,
    pub target_user: UserId,
}

impl DispatchAcceptFollowRequest {
    pub fn new(accepting_user: UserId, target_user: UserId) -> Self {
        DispatchAcceptFollowRequest {
            accepting_user,
            target_user,
        }
    }
}

impl ResponseType for DispatchAcceptFollowRequest {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchDenyFollowRequest {
    pub denying_user: UserId,
    pub target_user: UserId,
}

impl DispatchDenyFollowRequest {
    pub fn new(denying_user: UserId, target_user: UserId) -> Self {
        DispatchDenyFollowRequest {
            denying_user,
            target_user,
        }
    }
}

impl ResponseType for DispatchDenyFollowRequest {
    type Item = ();
    type Error = ();
}
