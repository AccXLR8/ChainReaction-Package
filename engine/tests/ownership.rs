#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn explosion_captures_opponent_cells() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);
    set_cell(&mut state, 1, 1, 1, 1);

    let result = apply_ok(&state, 0, mv(1, 0));

    assert_cell(&result.final_state, 1, 1, Some(0), 2);
}

#[test]
fn captured_cell_can_trigger_followup_explosion() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);
    set_cell(&mut state, 1, 1, 1, 3); // interior critical is 4

    let result = apply_ok(&state, 0, mv(1, 0));

    assert_eq!(
        result.timeline.steps.len(),
        2,
        "second wave should include captured cell"
    );
    assert_eq!(result.timeline.steps[1].explosions.len(), 1);
    let captured_index = idx(&result.final_state, 1, 1) as u32;
    assert_eq!(result.timeline.steps[1].explosions[0].cell, captured_index);
    assert_cell(&result.final_state, 1, 1, None, 0);
    assert_cell(&result.final_state, 1, 2, Some(0), 1);
    assert_cell(&result.final_state, 0, 1, Some(0), 1);
    assert_cell(&result.final_state, 2, 1, Some(0), 1);
}
