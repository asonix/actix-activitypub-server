use actix::{ResponseType, SyncAddress};

use super::{UserAddress, UserId, Users};

#[derive(Clone, Debug)]
pub struct Lookup(pub UserId);

#[derive(Clone, Debug)]
pub struct LookupMany(pub Vec<UserId>);

#[derive(Clone)]
pub struct NewUser(pub SyncAddress<Users>);

#[derive(Clone)]
pub struct NewUserFull(pub UserId, pub UserAddress);

#[derive(Clone)]
pub struct AnnounceNewUser(pub UserId, pub UserAddress);

#[derive(Clone, Debug)]
pub struct DeleteUser(pub UserId);

#[derive(Clone, Debug)]
pub struct UserSize;
