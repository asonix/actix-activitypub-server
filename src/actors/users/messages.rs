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

pub struct RequestPeers(pub SyncAddress<Users>);

impl ResponseType for RequestPeers {
    type Item = ();
    type Error = ();
}

pub struct ReplyPeers(pub Vec<SyncAddress<Users>>);

impl ResponseType for ReplyPeers {
    type Item = ();
    type Error = ();
}

pub struct RequestBackfill(pub SyncAddress<Users>, pub usize);

impl ResponseType for RequestBackfill {
    type Item = ();
    type Error = ();
}

pub struct ReplyBackfill(pub usize, pub Vec<(UserId, UserAddress)>);

impl ResponseType for ReplyBackfill {
    type Item = ();
    type Error = ();
}

pub struct PeerSize;

impl ResponseType for PeerSize {
    type Item = usize;
    type Error = ();
}

pub struct UserSize;

impl ResponseType for UserSize {
    type Item = usize;
    type Error = ();
}
