use std::collections::HashMap;

use actix::{Actor, Context, Handler};

use super::{Id, PostId};

pub mod messages;

use self::messages::*;

pub struct Post;

pub struct Posts {
    current_id: u64,
    posts: HashMap<PostId, Post>,
}

impl Posts {
    pub fn new() -> Self {
        Default::default()
    }

    fn generate_post_id(&mut self) -> PostId {
        let post_id = Id(self.current_id);

        self.current_id += 1;

        post_id
    }
}

impl Default for Posts {
    fn default() -> Self {
        Posts {
            current_id: 0,
            posts: HashMap::new(),
        }
    }
}

impl Actor for Posts {
    type Context = Context<Self>;
}

impl Handler<NewPost> for Posts {
    type Result = Result<PostId, ()>;

    fn handle(&mut self, _: NewPost, _: &mut Context<Self>) -> Self::Result {
        let post_id = self.generate_post_id();

        self.posts.insert(post_id, Post);

        Ok(post_id)
    }
}

impl Handler<DeletePost> for Posts {
    type Result = ();

    fn handle(&mut self, delete_post: DeletePost, _: &mut Context<Self>) -> Self::Result {
        let DeletePost(post_id) = delete_post;

        self.posts.remove(&post_id);
    }
}
