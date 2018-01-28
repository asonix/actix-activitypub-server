use actors::peered::{HandleAnnounce, HandleMessage, HandleMessageType};
use super::messages::*;
use super::post::Post;
use super::{PostId, Posts};

impl HandleMessage<NewPost> for Posts {
    type Broadcast = NewPostFull;
    type Item = PostId;
    type Error = ();

    fn handle_message(&mut self, msg: NewPost) -> HandleMessageType<PostId, (), NewPostFull> {
        let (post_id, post) = self.new_post(msg.0, msg.1);

        (Ok(post_id), Some(NewPostFull(post_id, post)))
    }
}

impl HandleMessage<DeletePost> for Posts {
    type Broadcast = DeletePost;
    type Item = ();
    type Error = ();

    fn handle_message(&mut self, msg: DeletePost) -> HandleMessageType<(), (), DeletePost> {
        self.delete_post(msg.0);

        (Ok(()), Some(msg))
    }
}

impl HandleMessage<GetPostsByIds> for Posts {
    type Broadcast = ();
    type Item = (Vec<Post>, Vec<PostId>);
    type Error = ();

    fn handle_message(&mut self, msg: GetPostsByIds) -> HandleMessageType<Self::Item, (), ()> {
        (Ok(self.get_posts(msg.0)), None)
    }
}

impl HandleMessage<PostSize> for Posts {
    type Broadcast = ();
    type Item = usize;
    type Error = ();

    fn handle_message(&mut self, _: PostSize) -> HandleMessageType<usize, (), ()> {
        (Ok(self.posts.len()), None)
    }
}

impl HandleAnnounce<NewPostFull> for Posts {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: NewPostFull) -> Result<(), ()> {
        self.posts.insert(msg.0, msg.1);
        Ok(())
    }
}

impl HandleAnnounce<DeletePost> for Posts {
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: DeletePost) -> Result<(), ()> {
        self.delete_post(msg.0);
        Ok(())
    }
}
