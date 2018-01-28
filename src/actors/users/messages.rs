use std::collections::BTreeSet;

use actix::SyncAddress;

use super::{UserAddress, UserId, Users};
use actors::peered::Peered;

#[derive(Clone, Debug)]
pub struct Lookup(pub UserId);

#[derive(Clone, Debug)]
pub struct LookupMany(pub BTreeSet<UserId>);

#[derive(Clone)]
pub struct NewUser(pub SyncAddress<Peered<Users>>);

#[derive(Clone)]
pub struct NewUserFull(pub UserId, pub UserAddress);

#[derive(Clone)]
pub struct AnnounceNewUser(pub UserId, pub UserAddress);

#[derive(Clone, Debug)]
pub struct DeleteUser(pub UserId);

#[derive(Clone, Debug)]
pub struct UserSize;
