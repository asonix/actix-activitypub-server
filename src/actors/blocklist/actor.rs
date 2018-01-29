use actors::peered::{HandleAnnounce, HandleMessage, HandleMessageType};

use super::Blocklists;
use super::messages::*;

impl HandleMessage<Block> for Blocklists {
    type Broadcast = Block;
    type Item = ();
    type Error = ();

    fn handle_message(
        &mut self,
        message: Block,
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
        message: Unblock,
    ) -> HandleMessageType<Self::Item, Self::Error, Self::Broadcast> {
        self.unblock_user(msg.0, msg.1);

        (Ok(()), Some(msg))
    }
}
