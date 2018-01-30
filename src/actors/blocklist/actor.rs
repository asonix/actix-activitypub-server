use std::collections::HashSet;

use actors::peered::{HandleAnnounce, HandleMessage, HandleMessageType};

use super::{Blocklists, UserId};
use super::messages::*;

impl HandleMessage<Block> for Blocklists {
    type Broadcast = Block;
    type Item = ();
    type Error = ();

    fn handle_message(
        &mut self,
        msg: Block,
    ) -> HandleMessageType<Self::Item, Self::Error, Self::Broadcast> {
        self.block_user(msg.0, msg.1);

        (Ok(()), Some(msg))
    }
}

impl HandleMessage<Unblock> for Blocklists {
    type Broadcast = Unblock;
    type Item = ();
    type Error = ();

    fn handle_message(
        &mut self,
        msg: Unblock,
    ) -> HandleMessageType<Self::Item, Self::Error, Self::Broadcast> {
        self.unblock_user(msg.0, msg.1);

        (Ok(()), Some(msg))
    }
}

impl HandleMessage<GetBlocklist> for Blocklists {
    type Broadcast = ();
    type Item = HashSet<UserId>;
    type Error = ();

    fn handle_message(&mut self, msg: GetBlocklist) -> HandleMessageType<HashSet<UserId>, (), ()> {
        (Ok(self.get_blocked_users(msg.0)), None)
    }
}

impl HandleMessage<GetBlockedBy> for Blocklists {
    type Broadcast = ();
    type Item = HashSet<UserId>;
    type Error = ();

    fn handle_message(&mut self, msg: GetBlockedBy) -> HandleMessageType<HashSet<UserId>, (), ()> {
        (Ok(self.is_blocked_by(msg.0)), None)
    }
}

impl HandleMessage<CanSpeak> for Blocklists {
    type Broadcast = ();
    type Item = bool;
    type Error = ();

    fn handle_message(&mut self, msg: CanSpeak) -> HandleMessageType<bool, (), ()> {
        (Ok(self.can_interact(msg.0, msg.1)), None)
    }
}

impl HandleAnnounce<Block> for Blocklists {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: Block) -> Result<Self::Item, Self::Error> {
        self.block_user(msg.0, msg.1);

        Ok(())
    }
}

impl HandleAnnounce<Unblock> for Blocklists {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: Unblock) -> Result<Self::Item, Self::Error> {
        self.unblock_user(msg.0, msg.1);

        Ok(())
    }
}
