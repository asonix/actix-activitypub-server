use std::collections::BTreeMap;
use std::cmp::Ordering;

use actix::{Actor, AsyncContext, Context, Handler, SyncAddress};

use super::{Id, PostId, PostsId, UserId};

pub mod messages;

use self::messages::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Post {
    post_id: PostId,
    author: UserId,
}

impl Ord for Post {
    fn cmp(&self, other: &Post) -> Ordering {
        self.post_id.cmp(&other.post_id)
    }
}

impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Post) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Posts {
    posts_id: PostsId,
    current_id: u64,
    posts: BTreeMap<PostId, Post>,
    redundancy: Vec<SyncAddress<Posts>>,
}

impl Posts {
    pub fn new(posts_id: PostsId) -> Self {
        Posts {
            posts_id: posts_id,
            current_id: 0,
            posts: BTreeMap::new(),
            redundancy: Vec::new(),
        }
    }

    pub fn add_peer(mut self, peer: SyncAddress<Posts>) -> Self {
        self.redundancy.push(peer);
        self
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

impl Actor for Posts {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        if !self.redundancy.is_empty() {
            self.redundancy[0].send(RequestPeers(ctx.address()));
            self.redundancy[0].send(RequestBackfill(ctx.address(), 0));
        }
    }
}

impl Handler<NewPost> for Posts {
    type Result = Result<PostId, ()>;

    fn handle(&mut self, msg: NewPost, _: &mut Context<Self>) -> Self::Result {
        let (post_id, post) = self.new_post(msg.0);

        for addr in &self.redundancy {
            addr.send(AnnounceNewPost(post_id, post.clone()));
        }

        Ok(post_id)
    }
}

impl Handler<DeletePost> for Posts {
    type Result = ();

    fn handle(&mut self, msg: DeletePost, _: &mut Context<Self>) -> Self::Result {
        self.delete_post(msg.0)
    }
}

impl Handler<GetPostsByIds> for Posts {
    type Result = Result<Vec<Post>, ()>;

    fn handle(&mut self, msg: GetPostsByIds, _: &mut Context<Self>) -> Self::Result {
        Ok(self.get_posts(msg.0))
    }
}

impl Handler<AnnounceNewPost> for Posts {
    type Result = ();

    fn handle(&mut self, msg: AnnounceNewPost, _: &mut Context<Self>) -> Self::Result {
        self.posts.insert(msg.0, msg.1);
    }
}

impl Handler<AnnounceDeletePost> for Posts {
    type Result = ();

    fn handle(&mut self, msg: AnnounceDeletePost, _: &mut Context<Self>) -> Self::Result {
        self.delete_post(msg.0);
    }
}

impl Handler<AnnounceRedundancy> for Posts {
    type Result = ();

    fn handle(&mut self, msg: AnnounceRedundancy, _: &mut Context<Self>) -> Self::Result {
        self.redundancy.push(msg.0);
    }
}

impl Handler<RequestPeers> for Posts {
    type Result = ();

    fn handle(&mut self, msg: RequestPeers, _: &mut Context<Self>) -> Self::Result {
        msg.0.send(ReplyPeers(self.redundancy.clone()));

        self.redundancy.push(msg.0);
    }
}

impl Handler<ReplyPeers> for Posts {
    type Result = ();

    fn handle(&mut self, msg: ReplyPeers, _: &mut Context<Self>) -> Self::Result {
        self.redundancy.extend(msg.0);
    }
}

impl Handler<RequestBackfill> for Posts {
    type Result = ();

    fn handle(&mut self, msg: RequestBackfill, _: &mut Context<Self>) -> Self::Result {
        let backfill = self.posts
            .iter()
            .skip(msg.1)
            .take(100)
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();

        msg.0.send(ReplyBackfill(msg.1, backfill));
    }
}

impl Handler<ReplyBackfill> for Posts {
    type Result = ();

    fn handle(&mut self, msg: ReplyBackfill, ctx: &mut Context<Self>) -> Self::Result {
        if msg.1.len() == 100 {
            self.redundancy
                .get(0)
                .map(|node| node.send(RequestBackfill(ctx.address(), msg.0 + 100)));
        }

        self.posts.extend(msg.1);
    }
}

impl Handler<PeerSize> for Posts {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: PeerSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.redundancy.len())
    }
}

impl Handler<PostSize> for Posts {
    type Result = Result<usize, ()>;

    fn handle(&mut self, _: PostSize, _: &mut Context<Self>) -> Self::Result {
        Ok(self.posts.len())
    }
}
