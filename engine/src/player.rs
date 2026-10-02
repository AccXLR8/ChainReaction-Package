//! Player identifiers and per-player runtime bookkeeping.

use serde::{Deserialize, Serialize};

/// Compact identifier assigned to each player when the game starts.
pub type PlayerId = u8;

/// Mutable player bookkeeping used inside `GameState`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerState {
    pub id: PlayerId,
    pub eliminated: bool,
    pub has_moved: bool,
}

impl PlayerState {
    /// Creates a fresh player entry flagged as alive and not yet moved.
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            eliminated: false,
            has_moved: false,
        }
    }
}
