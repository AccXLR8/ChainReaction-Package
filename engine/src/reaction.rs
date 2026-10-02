//! Types describing the ordered chain-reaction timeline returned to callers.

use crate::{board::CellIndex, player::PlayerId};
use serde::{Deserialize, Serialize};

/// Ordered list of simultaneous reaction waves for a single applied move.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReactionTimeline {
    pub placement: OrbPlacement,
    pub steps: Vec<ReactionStep>,
}

impl ReactionTimeline {
    /// Creates an empty timeline anchored to the player's initial placement.
    pub fn new(placement: OrbPlacement) -> Self {
        Self {
            placement,
            steps: Vec::new(),
        }
    }

    /// Returns true if no reaction waves were recorded.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Records which player placed an orb and in which cell to start the simulation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrbPlacement {
    pub cell: CellIndex,
    pub player: PlayerId,
}

/// Set of simultaneous explosions and transfers that occur within a single wave.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReactionStep {
    pub explosions: Vec<Explosion>,
    pub transfers: Vec<OrbTransfer>,
}

/// Explosion event for a specific cell owned by a given player.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Explosion {
    pub cell: CellIndex,
    pub player: PlayerId,
}

/// Directed orb movement from one cell to another during a wave.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrbTransfer {
    pub from: CellIndex,
    pub to: CellIndex,
    pub player: PlayerId,
}
