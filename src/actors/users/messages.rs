use actix::ResponseType;

use super::{UserAddress, UserId};

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
