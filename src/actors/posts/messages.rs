use std::collections::BTreeSet;

use actix::ResponseType;

use super::{Post, PostId, UserId};

#[derive(Clone, Debug)]
pub struct NewPost(pub UserId, pub BTreeSet<UserId>);

#[derive(Clone, Copy, Debug)]
pub struct DeletePost(pub PostId);

impl ResponseType for DeletePost {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Debug)]
pub struct GetPostsByIds(pub Vec<PostId>);

#[derive(Clone, Debug)]
pub struct NewPostFull(pub PostId, pub Post);

#[derive(Clone, Copy, Debug)]
pub struct PostSize;
