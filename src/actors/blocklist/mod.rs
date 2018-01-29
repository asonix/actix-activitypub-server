use std::collections::{BTreeMap, HashSet};

use actors::peered::PeeredInner;
use super::UserId;

mod actor;
pub mod messages;

pub struct Blocklists {
    lists: BTreeMap<UserId, HashSet<UserId>>,
    inverses: BTreeMap<UserId, HashSet<UserId>>,
}

impl Blocklists {
    pub fn new() -> Self {
        Blocklists::default()
    }

    fn block_user(&mut self, active_user: UserId, blocked_user: UserId) {
        self.lists.entry(active_user)
            .or_insert(HashSet::new())
            .insert(blocked_user);

        self.inverses.entry(blocked_user)
            .or_insert(HashSet::new())
            .insert(active_user);
    }

    fn unblock_user(&mut self, active_user: UserId, unblocked_user: UserId) {
        let is_empty = self.lists.get_mut(&active_user).map(|list| {
            list.remove(&unblocked_user);

            list.is_empty()
        });

        if let Some(true) = is_empty {
            self.lists.remove(&active_user);
        }

        let is_empty = self.inverses.get_mut(&blocked_user).map(|inverse| {
            inverse.remove(&active_user);

            inverse.is_empty()
        });

        if let Some(true) = is_empty {
            self.inverses.remove(&blocked_user);
        }
    }

    fn get_blocked_users(&self, user_id: UserId) -> HashSet<UserId> {
        self.lists.get(&user_id).cloned().unwrap_or(HashSet::new())
    }

    fn is_blocked_by(&self, user_id: UserId) -> HashSet<UserId> {
        self.inverses.get(&user_id).cloned().unwrap_or(HashSet::new())
    }
}

impl Default for Blocklists {
    fn default() -> Self {
        Blocklists {
            lists: BTreeMap::new(),
            inverses: BTreeMap::new(),
        }
    }
}

impl PeeredInner for Blocklists {
    type Backfill = BTreeMap<UserId, HashSet<UserId>>;
    type Request = usize;

    fn backfill(&self, req: Self::Request) -> Self::Backfill {
        self.lists.iter().skip(req).take(10).cloned().collect()
    }

    fn backfill_init(&self) -> Self::Request {
        0
    }

    fn handle_backfill(&mut self, backfill: Self::Backfill) -> Option<Self::Request> {
        for (user, blocklist) in backfill {
            for blocked_user in &blocklist {
                self.inverses.entry(*blocked_user)
                    .or_insert(HashSet::new())
                    .insert(user);
            }

            self.lists.entry(backfill)
                .or_insert(HashSet::new())
                .extend(blocklist);
        }
    }
}
