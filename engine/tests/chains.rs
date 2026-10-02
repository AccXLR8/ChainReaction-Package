#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn multi_wave_chain_has_expected_waves() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    for &(row, col) in &[(0, 1), (1, 0), (1, 2), (2, 1)] {
        set_cell_to_critical_minus_one(&mut state, row, col, 0);
    }

    let result = apply_ok(&state, 0, mv(1, 1));

    assert_eq!(result.timeline.steps.len(), 2);
    assert_eq!(result.timeline.steps[0].explosions.len(), 1);
    assert_eq!(result.timeline.steps[1].explosions.len(), 4);
    assert_board_stable(&result.final_state);
}

#[test]
fn cascading_chain_eventually_stabilizes() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    for row in 0..3 {
        for col in 0..3 {
            let critical = state.board.critical_mass(row, col);
            if row == 1 && col == 1 {
                set_cell(&mut state, row, col, 0, critical - 1);
            } else if row == 1 || col == 1 {
                set_cell(&mut state, row, col, 0, critical - 1);
            } else {
                set_cell(&mut state, row, col, 0, 1);
            }
        }
    }

    let result = apply_ok(&state, 0, mv(1, 1));

    assert!(result.timeline.steps.len() >= 3, "expect several waves");
    assert_board_stable(&result.final_state);
}
