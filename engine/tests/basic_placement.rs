#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn placing_on_empty_cell_assigns_owner_and_orb() {
    let state = fresh_game(5, 5, &[0, 1]);
    let result = apply_ok(&state, 0, mv(1, 1));
    assert_cell(&result.final_state, 1, 1, Some(0), 1);
    assert_eq!(result.timeline.steps.len(), 0, "no reactions expected");
    assert_eq!(result.final_state.turn_number, 1);
    assert_eq!(result.final_state.current_player(), 1);
    assert_board_stable(&result.final_state);
}

#[test]
fn placing_on_own_cell_increments_count_without_exploding() {
    let state = fresh_game(4, 4, &[0, 1]);
    let first = apply_ok(&state, 0, mv(1, 1));
    assert_cell(&first.final_state, 1, 1, Some(0), 1);
    let second = apply_ok(&first.final_state, 1, mv(2, 2));
    let third = apply_ok(&second.final_state, 0, mv(1, 1));
    assert_cell(&third.final_state, 1, 1, Some(0), 2);
    assert_eq!(third.timeline.steps.len(), 0);
    assert_board_stable(&third.final_state);
}

#[test]
fn turn_rotation_follows_a_b_a_cycle() {
    let state = fresh_game(3, 3, &[0, 1]);
    let after_a = apply_ok(&state, 0, mv(0, 0));
    assert_eq!(after_a.final_state.current_player(), 1);
    let after_b = apply_ok(&after_a.final_state, 1, mv(0, 1));
    assert_eq!(after_b.final_state.current_player(), 0);
    assert_eq!(after_b.final_state.turn_number, 2);
}
