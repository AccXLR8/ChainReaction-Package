//! Result types returned from `apply_move`, including final state and wave timeline.

use crate::{game_state::GameState, player::PlayerId, reaction::ReactionTimeline};
use serde::{Deserialize, Serialize};

/// Wrapper returned by `apply_move`, bundling the new state and metadata.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveResult {
    pub final_state: GameState,
    pub timeline: ReactionTimeline,
    pub outcome: GameResult,
}

/// Summary of whether the game is still active or who won.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameResult {
    pub status: crate::game_state::GameStatus,
    pub winner: Option<PlayerId>,
    pub losers: Vec<PlayerId>,
}

impl GameResult {
    /// Helper for the common "game still running" case.
    pub fn active() -> Self {
        Self {
            status: crate::game_state::GameStatus::Active,
            winner: None,
            losers: Vec::new(),
        }
    }

    /// Helper for the terminal case where a single winner remains.
    pub fn won(winner: PlayerId, losers: Vec<PlayerId>) -> Self {
        Self {
            status: crate::game_state::GameStatus::Won {
                winner,
                eliminated: losers.clone(),
            },
            winner: Some(winner),
            losers,
        }
    }
}
