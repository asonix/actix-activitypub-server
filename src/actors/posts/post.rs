use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::{PostId, UserId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Post {
    pub post_id: PostId,
    pub author: UserId,
    pub mentions: BTreeSet<UserId>,
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
