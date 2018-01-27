use actix::{ResponseType, SyncAddress};

use super::{Post, PostId, Posts, UserId};

pub struct NewPost(pub UserId);

impl ResponseType for NewPost {
    type Item = PostId;
    type Error = ();
}

pub struct DeletePost(pub PostId);

impl ResponseType for DeletePost {
    type Item = ();
    type Error = ();
}

pub struct GetPostsByIds(pub Vec<PostId>);

impl ResponseType for GetPostsByIds {
    type Item = Vec<Post>;
    type Error = ();
}

pub struct AnnounceNewPost(pub PostId, pub Post);

impl ResponseType for AnnounceNewPost {
    type Item = ();
    type Error = ();
}

pub struct AnnounceDeletePost(pub PostId);

impl ResponseType for AnnounceDeletePost {
    type Item = ();
    type Error = ();
}

pub struct AnnounceRedundancy(pub SyncAddress<Posts>);

impl ResponseType for AnnounceRedundancy {
    type Item = ();
    type Error = ();
}

pub struct RequestPeers(pub SyncAddress<Posts>);

impl ResponseType for RequestPeers {
    type Item = ();
    type Error = ();
}

pub struct ReplyPeers(pub Vec<SyncAddress<Posts>>);

impl ResponseType for ReplyPeers {
    type Item = ();
    type Error = ();
}

pub struct RequestBackfill(pub SyncAddress<Posts>, pub usize);

impl ResponseType for RequestBackfill {
    type Item = ();
    type Error = ();
}

pub struct ReplyBackfill(pub usize, pub Vec<(PostId, Post)>);

impl ResponseType for ReplyBackfill {
    type Item = ();
    type Error = ();
}

pub struct PeerSize;

impl ResponseType for PeerSize {
    type Item = usize;
    type Error = ();
}

pub struct PostSize;

impl ResponseType for PostSize {
    type Item = usize;
    type Error = ();
}
