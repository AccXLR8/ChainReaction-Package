//! Deterministic Chain Reaction game engine core library.

mod board;
mod cell;
mod engine;
mod error;
mod game_state;
mod mv;
mod player;
mod reaction;
mod result;

pub use board::{Board, CellIndex, Direction};
pub use engine::{apply_move, new_game, replay, validate_move};
pub use error::{EngineError, IllegalMove, InvalidBoard, InvalidState, ReplayError};
pub use game_state::{GameState, GameStatus};
pub use mv::Move;
pub use player::{PlayerId, PlayerState};
pub use reaction::{Explosion, OrbPlacement, OrbTransfer, ReactionStep, ReactionTimeline};
pub use result::{GameResult, MoveResult};
