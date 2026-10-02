//! Minimal cell representation: owner tracking plus orb count helpers.

use crate::player::PlayerId;
use serde::{Deserialize, Serialize};

/// Single board cell storing its owner and the number of orbs present.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cell {
    pub owner: Option<PlayerId>,
    pub orb_count: u32,
}

impl Cell {
    /// Creates a new empty cell with no owner and zero orbs.
    pub const fn empty() -> Self {
        Self {
            owner: None,
            orb_count: 0,
        }
    }

    /// Returns true if the cell currently holds no orbs/owner.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.owner.is_none()
    }

    /// Increments the orb count and transfers ownership to the provided player.
    #[inline]
    pub fn add_orb(&mut self, owner: PlayerId) {
        self.owner = Some(owner);
        self.orb_count += 1;
    }

    /// Removes up to `count` orbs, clearing ownership if the cell empties out.
    #[inline]
    pub fn remove_orbs(&mut self, count: u32) {
        self.orb_count = self.orb_count.saturating_sub(count);
        if self.orb_count == 0 {
            self.owner = None;
        }
    }

    /// Empties the cell entirely and returns the number of removed orbs.
    #[inline]
    pub fn drain(&mut self) -> u32 {
        let removed = self.orb_count;
        self.orb_count = 0;
        self.owner = None;
        removed
    }
}
