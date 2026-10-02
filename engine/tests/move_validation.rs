#[path = "common/mod.rs"]
mod common;

use common::*;
use engine::{apply_move, validate_move, GameStatus, IllegalMove};

#[test]
fn move_out_of_bounds_is_rejected_without_mutation() {
    let state = fresh_game(3, 3, &[0, 1]);
    let snapshot_before = snapshot(&state);
    let err = validate_move(&state, 0, mv(9, 9)).expect_err("should be out of bounds");
    assert!(matches!(err, IllegalMove::OutOfBounds));
    assert_eq!(
        snapshot(&state),
        snapshot_before,
        "state mutated on invalid move"
    );
}

#[test]
fn cannot_move_when_not_players_turn() {
    let state = fresh_game(3, 3, &[0, 1]);
    let err = validate_move(&state, 1, mv(0, 0)).expect_err("wrong player's turn");
    assert!(matches!(err, IllegalMove::NotPlayersTurn));
}

#[test]
fn cannot_move_into_opponent_owned_cell() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell(&mut state, 0, 0, 0, 1);
    let err = validate_move(&state, 1, mv(0, 0)).expect_err("cell owned by player 0");
    assert!(matches!(err, IllegalMove::CellOwnedByOpponent { owner: 0 }));
}

#[test]
fn finished_game_rejects_further_moves() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    state.status = GameStatus::Won {
        winner: 0,
        eliminated: vec![1],
    };
    let err = validate_move(&state, 0, mv(0, 0)).expect_err("game already finished");
    assert!(matches!(err, IllegalMove::GameAlreadyFinished));
}

#[test]
fn eliminated_player_cannot_move() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    state.players[0].eliminated = true;
    let err = validate_move(&state, 0, mv(0, 0)).expect_err("player 0 eliminated");
    assert!(matches!(err, IllegalMove::PlayerEliminated));
}

#[test]
fn invalid_move_does_not_change_state_when_using_apply_move() {
    let state = fresh_game(3, 3, &[0, 1]);
    let snapshot_before = snapshot(&state);
    let err = apply_move(&state, 0, mv(99, 99)).expect_err("should error");
    match err {
        engine::EngineError::IllegalMove(IllegalMove::OutOfBounds) => {}
        other => panic!("unexpected error {other:?}"),
    }
    assert_eq!(snapshot(&state), snapshot_before);
}
