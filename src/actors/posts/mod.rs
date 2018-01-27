use std::collections::BTreeMap;

use actix::{Actor, AsyncContext, Context, Handler, SyncAddress};

use super::{Id, PostId, PostsId, UserId};
use super::peered::{PeeredInner, HandleMessage, HandleAnnounce};

pub mod messages;
mod post;

use self::messages::*;
pub use self::post::Post;

const BACKFILL_CHUNK_SIZE: usize = 100;

pub struct Posts {
    posts_id: PostsId,
    current_id: u64,
    posts: BTreeMap<PostId, Post>,
}

impl Posts {
    pub fn new(posts_id: PostsId) -> Self {
        Posts {
            posts_id: posts_id,
            current_id: 0,
            posts: BTreeMap::new(),
        }
    }

    fn generate_post_id(&mut self) -> PostId {
        let post_id = Id(self.current_id);

        self.current_id += 1;

        PostId::new(self.posts_id, post_id)
    }

    fn new_post(&mut self, author: UserId) -> (PostId, Post) {
        let post_id = self.generate_post_id();
        let post = Post { post_id, author };

        self.add_post(post_id, post.clone());

        (post_id, post)
    }

    fn add_post(&mut self, post_id: PostId, post: Post) {
        self.posts.insert(post_id, post);
    }

    fn delete_post(&mut self, post_id: PostId) {
        self.posts.remove(&post_id);
    }

    fn get_posts(&mut self, post_ids: Vec<PostId>) -> Vec<Post> {
        let (posts, _) = post_ids.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut posts, mut missing), post_id| {
                match self.posts.get(&post_id).cloned() {
                    Some(post) => posts.push(post),
                    None => missing.push(post_id),
                }

                (posts, missing)
            },
        );

        posts
    }
}

impl PeeredInner for Posts {
    type Backfill = (usize, Vec<Post>);
    type Request = usize;

    fn backfill(&self, req: Self::Request) -> Self::Backfill {
        let p = self.posts
            .iter()
            .skip(req)
            .take(BACKFILL_CHUNK_SIZE)
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();

        (req, p)
    }

    fn backfill_init(&self) -> Self::Request {
        0
    }

    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request> {
        let mut ret = None;

        if backfill.1.len() == BACKFILL_CHUNK_SIZE {
            ret = Some(backfill.0 + BACKFILL_CHUNK_SIZE);
        }

        self.posts.extend(backfill.1);

        ret
    }
}

impl HandleMessage for Posts {
    type Message = NewPost;
    type Broadcast = NewPostFull;
    type Item = PostId;
    type Error = ();

    fn handle_message(&mut self, msg: Self::Message) -> Self::Result {
        let (post_id, post) = self.new_post(msg.0);

        (Ok(post_id), Some(NewPostFull(post_id, post)))
    }
}

impl HandleMessage for Posts {
    type Message = DeletePost;
    type Broadcast = DeletePost;
    type Item = ();
    type Error = ();

    fn handle_message(&mut self, msg: Self::Message) -> Self::Result {
        self.delete_post(msg.0);

        (Ok(()), Some(msg))
    }
}

impl HandleMessage for Posts {
    type Message = GetPostsByIds;
    type Broadcast = ();
    type Item = Vec<Post>;
    type Error = ();

    fn handle_message(&mut self, msg: Self::Message) -> Self::Result {
        (Ok(self.get_posts()), None)
    }
}

impl HandleMessage for Posts {
    type Message = PostSize;
    type Broadcast = ();
    type Item = usize;
    type Error = ();

    fn handle_message(&mut self, _: Self::Message) -> Self::Result {
        (Ok(self.posts.len()), None)
    }
}

impl HandleAnnounce for Posts {
    type Broadcast = NewPostFull;
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: Self::Broadcast) -> Self::Result {
        self.posts.insert(msg.0, msg.1);
        Ok(())
    }
}

impl HandleAnnounce for Posts {
    type Broadcast = DeletePost;
    type Item = ();
    type Error = ();

    fn handle_announce(&mut self, msg: Self::Broadcast) -> Self::Result {
        self.delete_post(msg.0);
    }
}
