use actors::peered::{HandleAnnounce, HandleMessage, HandleMessageType};
use super::messages::*;
use super::{UserAddress, UserId, Users};

impl HandleMessage<Lookup> for Users {
    type Broadcast = ();
    type Item = UserAddress;
    type Error = ();

    fn handle_message(&mut self, msg: Lookup) -> HandleMessageType<UserAddress, (), ()> {
        (self.get_user(msg.0).ok_or(()), None)
    }
}

impl HandleMessage<LookupMany> for Users {
    type Broadcast = ();
    type Item = (Vec<UserAddress>, Vec<UserId>);
    type Error = ();

    fn handle_message(&mut self, msg: LookupMany) -> HandleMessageType<Self::Item, (), ()> {
        (Ok(self.get_users(msg.0)), None)
    }
}

impl HandleMessage<NewUser> for Users {
    type Broadcast = NewUserFull;
    type Item = UserId;
    type Error = ();

    fn handle_message(
        &mut self,
        msg: NewUser,
    ) -> HandleMessageType<Self::Item, (), Self::Broadcast> {
        let (user_id, user_address) = self.new_user(msg.0);

        (Ok(user_id), Some(NewUserFull(user_id, user_address)))
    }
}

impl HandleMessage<DeleteUser> for Users {
    type Broadcast = DeleteUser;
    type Item = ();
    type Error = ();

    fn handle_message(&mut self, msg: DeleteUser) -> HandleMessageType<(), (), DeleteUser> {
        self.delete_user(msg.0);

        (Ok(()), Some(msg))
    }
}

impl HandleMessage<UserSize> for Users {
    type Broadcast = ();
    type Item = usize;
    type Error = ();

    fn handle_message(&mut self, _: UserSize) -> HandleMessageType<usize, (), ()> {
        (Ok(self.users.len()), None)
    }
}

impl HandleAnnounce<NewUserFull> for Users {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: NewUserFull) -> Result<(), ()> {
        self.add_user(msg.0, msg.1);
        Ok(())
    }
}

impl HandleAnnounce<DeleteUser> for Users {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: DeleteUser) -> Result<(), ()> {
        self.delete_user(msg.0);
        Ok(())
    }
}
