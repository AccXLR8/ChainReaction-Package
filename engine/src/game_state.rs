//! Authoritative game snapshot including board, players, and turn metadata.

use crate::{
    board::Board,
    error::InvalidState,
    player::{PlayerId, PlayerState},
};
use serde::{Deserialize, Serialize};

/// Complete, serializable snapshot of a running game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub players: Vec<PlayerState>,
    pub current_player_index: usize,
    pub turn_number: u64,
    pub status: GameStatus,
}

/// Minimal status indicator capturing whether the game is finished.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameStatus {
    Active,
    Won {
        winner: PlayerId,
        eliminated: Vec<PlayerId>,
    },
}

impl GameState {
    /// Builds a new state from a board and ordered player list.
    pub fn new(board: Board, players: Vec<PlayerState>) -> Result<Self, InvalidState> {
        if players.is_empty() {
            return Err(InvalidState::NoPlayers);
        }
        Ok(Self {
            board,
            players,
            current_player_index: 0,
            turn_number: 0,
            status: GameStatus::Active,
        })
    }

    /// Returns the ID of the player whose turn it currently is.
    #[inline]
    pub fn current_player(&self) -> PlayerId {
        self.players[self.current_player_index].id
    }

    /// Finds the next non-eliminated player index after `start`, wrapping around.
    pub fn next_active_player_index(&self, start: usize) -> Option<usize> {
        let player_count = self.players.len();
        for offset in 1..=player_count {
            let index = (start + offset) % player_count;
            if !self.players[index].eliminated {
                return Some(index);
            }
        }
        None
    }

    /// Returns the list of players still alive in turn order.
    pub fn living_players(&self) -> Vec<PlayerId> {
        self.players
            .iter()
            .filter(|p| !p.eliminated)
            .map(|p| p.id)
            .collect()
    }
}
