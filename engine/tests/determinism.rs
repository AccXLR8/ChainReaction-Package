#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn identical_state_and_move_yield_identical_results() {
    let mut state = fresh_game(4, 4, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 0, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);
    set_cell_to_critical_minus_one(&mut state, 2, 1, 0);

    let result_a = apply_ok(&state, 0, mv(1, 1));
    let result_b = apply_ok(&state, 0, mv(1, 1));
    let result_c = apply_ok(&state, 0, mv(1, 1));

    let ser_a = serde_json::to_string(&result_a).unwrap();
    let ser_b = serde_json::to_string(&result_b).unwrap();
    let ser_c = serde_json::to_string(&result_c).unwrap();
    assert_eq!(ser_a, ser_b);
    assert_eq!(ser_b, ser_c);
}

#[test]
fn determinism_holds_after_sequence_of_moves() {
    let game = fresh_game(5, 5, &[0, 1]);
    let moves = [mv(0, 0), mv(1, 1), mv(0, 0), mv(2, 2), mv(0, 1), mv(2, 1)];

    let mut state_a = game.clone();
    let mut state_b = game.clone();

    for mv in moves {
        let pa = state_a.current_player();
        state_a = apply_ok(&state_a, pa, mv).final_state;
        let pb = state_b.current_player();
        state_b = apply_ok(&state_b, pb, mv).final_state;
    }

    assert_eq!(snapshot(&state_a), snapshot(&state_b));
}
