//! High-level game orchestration: move validation, simulation, and replay helpers.

use crate::{
    board::{Board, CellIndex, Direction},
    error::{EngineError, IllegalMove, InvalidState, ReplayError},
    game_state::{GameState, GameStatus},
    mv::Move,
    player::{PlayerId, PlayerState},
    reaction::{Explosion, OrbPlacement, OrbTransfer, ReactionStep, ReactionTimeline},
    result::{GameResult, MoveResult},
};

/// Creates a fresh `GameState` with the requested board and turn order.
pub fn new_game(width: u16, height: u16, players: Vec<PlayerId>) -> Result<GameState, EngineError> {
    if players.len() < 2 {
        return Err(EngineError::InvalidState(InvalidState::NoPlayers));
    }
    let mut seen = std::collections::HashSet::new();
    for &player in &players {
        if !seen.insert(player) {
            return Err(EngineError::InvalidState(InvalidState::InvalidPlayerOrder));
        }
    }
    let board = Board::new(width, height)?;
    let player_states: Vec<PlayerState> = players.into_iter().map(PlayerState::new).collect();
    Ok(GameState::new(board, player_states)?)
}

/// Checks whether `player` may legally play `mv` in `state`.
pub fn validate_move(state: &GameState, player: PlayerId, mv: Move) -> Result<(), IllegalMove> {
    if !matches!(state.status, GameStatus::Active) {
        return Err(IllegalMove::GameAlreadyFinished);
    }
    if state.current_player() != player {
        return Err(IllegalMove::NotPlayersTurn);
    }
    let player_state = state
        .players
        .iter()
        .find(|p| p.id == player)
        .ok_or(IllegalMove::PlayerEliminated)?;
    if player_state.eliminated {
        return Err(IllegalMove::PlayerEliminated);
    }
    let index = state
        .board
        .index(mv.row, mv.col)
        .ok_or(IllegalMove::OutOfBounds)?;
    if let Some(owner) = state.board.cell(index).owner {
        if owner != player {
            return Err(IllegalMove::CellOwnedByOpponent { owner });
        }
    }
    Ok(())
}

/// Applies a validated move, runs the deterministic chain reaction, and returns the snapshot.
pub fn apply_move(
    state: &GameState,
    player: PlayerId,
    mv: Move,
) -> Result<MoveResult, EngineError> {
    validate_move(state, player, mv).map_err(EngineError::from)?;
    let mut next_state = state.clone();
    let index = next_state
        .board
        .index(mv.row, mv.col)
        .ok_or(EngineError::IllegalMove(IllegalMove::OutOfBounds))?;
    next_state.turn_number = next_state.turn_number.wrapping_add(1);
    {
        let cell = next_state.board.cell_mut(index);
        cell.add_orb(player);
    }

    mark_player_moved(&mut next_state, player)?;

    let mut timeline = ReactionTimeline::new(OrbPlacement {
        cell: index as CellIndex,
        player,
    });

    let mut initial_unstable = Vec::new();
    let critical = next_state.board.critical_mass_index(index);
    if next_state.board.cell(index).orb_count >= critical {
        initial_unstable.push(index as CellIndex);
    }

    simulate_reactions(&mut next_state.board, &mut timeline, initial_unstable);
    update_eliminations(&mut next_state);

    let outcome = evaluate_result(&mut next_state);

    if matches!(next_state.status, GameStatus::Active) {
        advance_turn(&mut next_state);
    }

    Ok(MoveResult {
        final_state: next_state,
        timeline,
        outcome,
    })
}

/// Replays a sequence of moves from an initial state, returning the final board snapshot.
pub fn replay(initial_state: &GameState, moves: &[Move]) -> Result<GameState, ReplayError> {
    let mut state = initial_state.clone();
    for (idx, mv) in moves.iter().enumerate() {
        let player = state.current_player();
        match apply_move(&state, player, *mv) {
            Ok(result) => {
                state = result.final_state;
            }
            Err(EngineError::IllegalMove(err)) => {
                return Err(ReplayError::IllegalMove {
                    step: idx,
                    error: err,
                });
            }
            Err(other) => return Err(ReplayError::Engine(other)),
        }
    }
    Ok(state)
}

/// Marks that `player` has taken at least one turn to unlock elimination logic.
fn mark_player_moved(state: &mut GameState, player: PlayerId) -> Result<(), EngineError> {
    let player_state = state
        .players
        .iter_mut()
        .find(|p| p.id == player)
        .ok_or(EngineError::InvalidState(InvalidState::InvalidPlayerOrder))?;
    player_state.has_moved = true;
    Ok(())
}

/// Resolves all queued unstable cells in deterministic waves, recording the reaction timeline.
fn simulate_reactions(
    board: &mut Board,
    timeline: &mut ReactionTimeline,
    mut current_wave: Vec<CellIndex>,
) {
    if current_wave.is_empty() {
        return;
    }
    while !current_wave.is_empty() {
        current_wave.sort_unstable();
        let mut step = ReactionStep::default();
        let mut next_wave: Vec<CellIndex> = Vec::new();
        let mut next_flags = vec![false; board.len()];
        for &cell_index in &current_wave {
            let idx = cell_index as usize;
            let snapshot = board.cell(idx).clone();
            let owner = snapshot.owner.expect("unstable cell must be owned");
            step.explosions.push(Explosion {
                cell: cell_index,
                player: owner,
            });
            let critical = board.critical_mass_index(idx);
            board.cell_mut(idx).drain();
            for direction in Direction::ALL {
                if let Some(neighbor_idx) = board.neighbor(idx, direction) {
                    board.cell_mut(neighbor_idx).add_orb(owner);
                    let neighbor_cell = neighbor_idx as CellIndex;
                    step.transfers.push(OrbTransfer {
                        from: cell_index,
                        to: neighbor_cell,
                        player: owner,
                    });
                    let neighbor_critical = board.critical_mass_index(neighbor_idx);
                    if board.cell(neighbor_idx).orb_count >= neighbor_critical
                        && !next_flags[neighbor_idx]
                    {
                        next_flags[neighbor_idx] = true;
                        next_wave.push(neighbor_cell);
                    }
                }
            }
        }
        timeline.steps.push(step);
        current_wave = next_wave;
    }
}

/// Eliminates players who have already moved but no longer own any cells.
fn update_eliminations(state: &mut GameState) {
    for player in &mut state.players {
        if player.eliminated {
            continue;
        }
        if !player.has_moved {
            continue;
        }
        let owned = state.board.owner_cell_count(player.id);
        if owned == 0 {
            player.eliminated = true;
        }
    }
}

/// Determines whether the game is still active or if a single winner remains.
fn evaluate_result(state: &mut GameState) -> GameResult {
    let living: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| !p.eliminated)
        .map(|p| p.id)
        .collect();
    if living.len() == 1 {
        let winner = living[0];
        let losers: Vec<PlayerId> = state
            .players
            .iter()
            .filter(|p| p.id != winner)
            .map(|p| p.id)
            .collect();
        state.status = GameStatus::Won {
            winner,
            eliminated: losers.clone(),
        };
        GameResult::won(winner, losers)
    } else {
        state.status = GameStatus::Active;
        GameResult::active()
    }
}

/// Advances the turn pointer to the next non-eliminated player, if any.
fn advance_turn(state: &mut GameState) {
    if let Some(next_index) = state.next_active_player_index(state.current_player_index) {
        state.current_player_index = next_index;
    }
}
