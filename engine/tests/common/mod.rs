#![allow(dead_code)]

use engine::{apply_move, new_game, Board, GameState, Move, MoveResult, PlayerId};
use serde_json;

/// Creates a fresh `GameState` for the supplied players (in turn order).
pub fn fresh_game(width: u16, height: u16, players: &[PlayerId]) -> GameState {
    new_game(width, height, players.to_vec()).expect("game creation should succeed")
}

/// Convenience constructor for `Move` using row/column coordinates.
pub fn mv(row: u16, col: u16) -> Move {
    Move::new(row, col)
}

/// Returns the flat cell index for (row, col) and panics if out of bounds.
pub fn idx(state: &GameState, row: u16, col: u16) -> usize {
    state
        .board
        .index(row, col)
        .unwrap_or_else(|| panic!("cell ({row},{col}) should exist"))
}

/// Mutates a cell to the requested owner/count.
pub fn set_cell(state: &mut GameState, row: u16, col: u16, owner: PlayerId, count: u32) {
    let index = idx(state, row, col);
    let cell = state.board.cell_mut(index);
    cell.owner = Some(owner);
    cell.orb_count = count;
}

/// Removes ownership/orbs from a cell.
pub fn clear_cell(state: &mut GameState, row: u16, col: u16) {
    let index = idx(state, row, col);
    let cell = state.board.cell_mut(index);
    cell.owner = None;
    cell.orb_count = 0;
}

/// Charges the cell so that it is exactly at its critical mass threshold.
pub fn set_cell_to_critical(state: &mut GameState, row: u16, col: u16, owner: PlayerId) {
    let critical = state.board.critical_mass(row, col);
    set_cell(state, row, col, owner, critical);
}

/// Charges the cell to one less than critical (useful for staging chains).
pub fn set_cell_to_critical_minus_one(state: &mut GameState, row: u16, col: u16, owner: PlayerId) {
    let critical = state.board.critical_mass(row, col);
    set_cell(state, row, col, owner, critical.saturating_sub(1));
}

/// Asserts a cell is owned/count as expected.
pub fn assert_cell(state: &GameState, row: u16, col: u16, owner: Option<PlayerId>, count: u32) {
    let index = idx(state, row, col);
    let cell = state.board.cell(index);
    assert_eq!(cell.owner, owner, "unexpected owner at ({row},{col})");
    assert_eq!(
        cell.orb_count, count,
        "unexpected orb count at ({row},{col})"
    );
}

/// Ensures that every cell is currently below its critical mass.
pub fn assert_board_stable(state: &GameState) {
    for i in 0..state.board.len() {
        let cell = state.board.cell(i);
        let critical = state.board.critical_mass_index(i);
        assert!(
            cell.orb_count < critical,
            "cell {:?} exceeded stability: count={} critical={}",
            state.board.coords(i),
            cell.orb_count,
            critical
        );
    }
}

/// Applies a move for the requested player and returns the successful `MoveResult`.
pub fn apply_ok(state: &GameState, player: PlayerId, mv: Move) -> MoveResult {
    apply_move(state, player, mv).expect("move must succeed")
}

/// Like `apply_ok`, but accepts row/col directly.
pub fn apply_ok_rc(state: &GameState, player: PlayerId, row: u16, col: u16) -> MoveResult {
    apply_ok(state, player, mv(row, col))
}

/// Convenience helper that serializes a state for equality snapshots.
pub fn snapshot(state: &GameState) -> serde_json::Value {
    serde_json::to_value(state).expect("state should serialize")
}

/// Returns the neighbor coordinates for a board, useful for assertions.
pub fn neighbor_coords(board: &Board, index: usize) -> Vec<(u16, u16)> {
    board
        .neighbors(index)
        .map(|idx| board.coords(idx))
        .collect()
}
