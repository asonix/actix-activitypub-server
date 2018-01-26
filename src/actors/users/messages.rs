use actix::{ResponseType, SyncAddress};

use super::{UserAddress, UserId, Users};

pub struct Lookup(pub UserId);

impl ResponseType for Lookup {
    type Item = UserAddress;
    type Error = ();
}

pub struct LookupMany(pub Vec<UserId>);

impl ResponseType for LookupMany {
    type Item = (Vec<UserAddress>, Vec<UserId>);
    type Error = ();
}

pub struct NewUser;

impl ResponseType for NewUser {
    type Item = UserId;
    type Error = ();
}

pub struct AnnounceNewUser(pub UserId, pub UserAddress);

impl ResponseType for AnnounceNewUser {
    type Item = ();
    type Error = ();
}

pub struct AnnounceDeleteUser(pub UserId);

impl ResponseType for AnnounceDeleteUser {
    type Item = ();
    type Error = ();
}

pub struct AnnounceRedundancy(pub SyncAddress<Users>);

impl ResponseType for AnnounceRedundancy {
    type Item = ();
    type Error = ();
}

pub struct ReplyRedundancy(pub SyncAddress<Users>);

impl ResponseType for ReplyRedundancy {
    type Item = ();
    type Error = ();
}
