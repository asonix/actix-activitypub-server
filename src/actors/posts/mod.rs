use std::collections::{BTreeMap, BTreeSet};

use super::{Id, PostId, PostsId, UserId};
use super::peered::{HandleAnnounce, HandleMessage, HandleMessageType, PeeredInner};

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

    fn new_post(&mut self, author: UserId, mentions: BTreeSet<UserId>) -> (PostId, Post) {
        let post_id = self.generate_post_id();
        let post = Post {
            post_id,
            author,
            mentions,
        };

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
    type Backfill = (usize, Vec<(PostId, Post)>);
    type Request = usize;

    fn backfill(&self, req: Self::Request) -> Self::Backfill {
        let p = self.posts
            .iter()
            .skip(req)
            .take(BACKFILL_CHUNK_SIZE)
            .map(|(a, b)| (*a, b.clone()))
            .collect();

        (req, p)
    }

    fn backfill_init(&self) -> Self::Request {
        0
    }

    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request> {
        let ret = if backfill.1.len() == BACKFILL_CHUNK_SIZE {
            Some(backfill.0 + BACKFILL_CHUNK_SIZE)
        } else {
            None
        };

        self.posts.extend(backfill.1);

        ret
    }
}

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
    type Item = Vec<Post>;
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
