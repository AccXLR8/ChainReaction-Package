#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn corner_explosion_distributes_to_two_neighbors() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 0, 0, 0);
    let result = apply_ok(&state, 0, mv(0, 0));
    let explosion_index = idx(&result.final_state, 0, 0) as u32;
    assert_eq!(result.timeline.steps.len(), 1);
    let step = &result.timeline.steps[0];
    assert_eq!(step.explosions.len(), 1);
    assert_eq!(step.explosions[0].cell, explosion_index);
    assert_eq!(step.transfers.len(), 2);
    assert_cell(&result.final_state, 0, 0, None, 0);
    assert_cell(&result.final_state, 0, 1, Some(0), 1);
    assert_cell(&result.final_state, 1, 0, Some(0), 1);
    assert_board_stable(&result.final_state);
}

#[test]
fn edge_explosion_hits_three_neighbors_only() {
    let mut state = fresh_game(4, 4, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 0, 1, 0);
    let result = apply_ok(&state, 0, mv(0, 1));
    assert_eq!(result.timeline.steps.len(), 1);
    let step = &result.timeline.steps[0];
    assert_eq!(
        step.transfers.len(),
        3,
        "edge has three orthogonal neighbors"
    );
    assert_cell(&result.final_state, 0, 1, None, 0);
    assert_cell(&result.final_state, 0, 0, Some(0), 1);
    assert_cell(&result.final_state, 0, 2, Some(0), 1);
    assert_cell(&result.final_state, 1, 1, Some(0), 1);
    assert_board_stable(&result.final_state);
}

#[test]
fn center_explosion_affects_four_neighbors() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    let result = apply_ok(&state, 0, mv(1, 1));
    assert_eq!(result.timeline.steps.len(), 1);
    let step = &result.timeline.steps[0];
    assert_eq!(step.transfers.len(), 4);
    assert_cell(&result.final_state, 1, 1, None, 0);
    assert_cell(&result.final_state, 0, 1, Some(0), 1);
    assert_cell(&result.final_state, 1, 2, Some(0), 1);
    assert_cell(&result.final_state, 2, 1, Some(0), 1);
    assert_cell(&result.final_state, 1, 0, Some(0), 1);
    assert_board_stable(&result.final_state);
}

#[test]
fn diagonals_are_never_affected() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    // Give diagonals distinct ownership/count to ensure they are untouched.
    set_cell(&mut state, 0, 0, 1, 5);
    set_cell(&mut state, 0, 2, 1, 6);
    set_cell(&mut state, 2, 0, 1, 7);
    set_cell(&mut state, 2, 2, 1, 8);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);

    let result = apply_ok(&state, 0, mv(1, 1));

    assert_cell(&result.final_state, 0, 0, Some(1), 5);
    assert_cell(&result.final_state, 0, 2, Some(1), 6);
    assert_cell(&result.final_state, 2, 0, Some(1), 7);
    assert_cell(&result.final_state, 2, 2, Some(1), 8);
}
