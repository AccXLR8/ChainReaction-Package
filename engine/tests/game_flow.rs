#[path = "common/mod.rs"]
mod common;

use common::*;
use engine::{apply_move, GameStatus, IllegalMove};

#[test]
fn player_not_eliminated_before_first_move() {
    let state = fresh_game(3, 3, &[0, 1]);
    let result = apply_ok(&state, 0, mv(0, 0));
    let player1 = &result.final_state.players[1];
    assert!(!player1.has_moved);
    assert!(
        !player1.eliminated,
        "player should remain active until they play"
    );
}

#[test]
fn player_eliminated_after_losing_cells_post_move() {
    let state = fresh_game(2, 2, &[0, 1]);
    let after_p0 = apply_ok(&state, 0, mv(0, 0));
    let after_p1 = apply_ok(&after_p0.final_state, 1, mv(1, 0));
    let final_result = apply_ok(&after_p1.final_state, 0, mv(0, 0));
    assert!(final_result.final_state.players[1].eliminated);
    assert!(matches!(
        final_result.outcome.status,
        GameStatus::Won { .. }
    ));
}

#[test]
fn player_two_can_eliminate_player_one_and_win() {
    let state = fresh_game(2, 2, &[0, 1]);
    let after_p0 = apply_ok(&state, 0, mv(0, 0));
    let after_p1 = apply_ok(&after_p0.final_state, 1, mv(1, 0));
    let after_p0_again = apply_ok(&after_p1.final_state, 0, mv(0, 1));
    let winning = apply_ok(&after_p0_again.final_state, 1, mv(1, 0));
    assert!(
        winning.final_state.players[0].eliminated,
        "player 0 should now be eliminated"
    );
    assert!(matches!(winning.outcome.status, GameStatus::Won { winner, .. } if winner == 1));
}

#[test]
fn winner_declared_and_follow_up_move_rejected() {
    let state = fresh_game(2, 2, &[0, 1]);
    let after_p0 = apply_ok(&state, 0, mv(0, 0));
    let after_p1 = apply_ok(&after_p0.final_state, 1, mv(1, 0));
    let winning = apply_ok(&after_p1.final_state, 0, mv(0, 0));
    assert!(matches!(winning.outcome.status, GameStatus::Won { .. }));

    let err = apply_move(&winning.final_state, 1, mv(1, 1)).expect_err("game finished");
    assert!(matches!(
        err,
        engine::EngineError::IllegalMove(IllegalMove::GameAlreadyFinished)
    ));
}
