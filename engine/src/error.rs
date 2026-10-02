//! Structured error reporting for the engine's public API and helpers.

use crate::{board::CellIndex, player::PlayerId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Top-level error envelope returned by public engine APIs.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("illegal move: {0}")]
    IllegalMove(#[from] IllegalMove),
    #[error("invalid board: {0}")]
    InvalidBoard(#[from] InvalidBoard),
    #[error("invalid state: {0}")]
    InvalidState(#[from] InvalidState),
}

/// Specific reasons why a move cannot be applied.
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IllegalMove {
    #[error("game already finished")]
    GameAlreadyFinished,
    #[error("not this player's turn")]
    NotPlayersTurn,
    #[error("player eliminated")]
    PlayerEliminated,
    #[error("coordinates out of bounds")]
    OutOfBounds,
    #[error("cell owned by another player")]
    CellOwnedByOpponent { owner: PlayerId },
    #[error("invalid cell index {0}")]
    InvalidCellIndex(CellIndex),
}

/// Internal consistency issues detected when constructing or mutating `GameState`.
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvalidState {
    #[error("game has no players")]
    NoPlayers,
    #[error("player order invalid")]
    InvalidPlayerOrder,
}

/// Validation errors specific to `Board` construction.
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvalidBoard {
    #[error("board dimension cannot be zero")]
    ZeroDimension,
    #[error("board dimension must be at least 2x2")]
    TooSmall,
}

/// Errors encountered while replaying a saved move list.
#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("engine error: {0}")]
    Engine(#[from] EngineError),
    #[error("illegal move at step {step}: {error}")]
    IllegalMove { step: usize, error: IllegalMove },
}
