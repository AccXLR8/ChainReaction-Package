//! Lightweight representation of a player's intent to place an orb.

use serde::{Deserialize, Serialize};

/// Absolute board coordinates for a single player action.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Move {
    pub row: u16,
    pub col: u16,
}

impl Move {
    /// Convenience constructor used by tests and callers.
    pub const fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}
