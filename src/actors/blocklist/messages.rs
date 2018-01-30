use super::UserId;

/// Block(acting_user, blocked_user)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Block(pub UserId, pub UserId);

/// Unblock(acting_user, blocked_user)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Unblock(pub UserId, pub UserId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GetBlocklist(pub UserId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GetBlockedBy(pub UserId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CanSpeak(pub UserId, pub UserId);
