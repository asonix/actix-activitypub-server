use actix::ResponseType;

use super::PostId;

pub struct NewPost;

impl ResponseType for NewPost {
    type Item = PostId;
    type Error = ();
}

pub struct DeletePost(pub PostId);

impl ResponseType for DeletePost {
    type Item = ();
    type Error = ();
}
