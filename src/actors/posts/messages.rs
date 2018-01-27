use actix::{ResponseType, SyncAddress};

use super::{Post, PostId, Posts, UserId};

#[derive(Clone, Debug)]
pub struct NewPost(pub UserId);

#[derive(Clone, Debug)]
pub struct DeletePost(pub PostId);

#[derive(Clone, Debug)]
pub struct GetPostsByIds(pub Vec<PostId>);

#[derive(Clone, Debug)]
pub struct NewPostFull(pub PostId, pub Post);

#[derive(Clone, Debug)]
pub struct PostSize;
