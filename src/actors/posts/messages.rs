use std::collections::BTreeSet;

use super::{Post, PostId, UserId};

#[derive(Clone, Debug)]
pub struct NewPost(pub UserId, pub BTreeSet<UserId>);

#[derive(Clone, Debug)]
pub struct DeletePost(pub PostId);

#[derive(Clone, Debug)]
pub struct GetPostsByIds(pub Vec<PostId>);

#[derive(Clone, Debug)]
pub struct NewPostFull(pub PostId, pub Post);

#[derive(Clone, Debug)]
pub struct PostSize;
