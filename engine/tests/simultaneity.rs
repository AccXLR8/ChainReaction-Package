#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn simultaneous_unstable_cells_share_wave() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 0, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);
    // Right and bottom remain low so they stay stable after wave 1

    let result = apply_ok(&state, 0, mv(1, 1));

    assert_eq!(result.timeline.steps.len(), 2);
    let second_wave = &result.timeline.steps[1];
    assert_eq!(
        second_wave.explosions.len(),
        2,
        "both edge cells explode together"
    );
    let cells: Vec<_> = second_wave.explosions.iter().map(|e| e.cell).collect();
    let mut sorted = cells.clone();
    sorted.sort();
    assert_eq!(cells, sorted, "explosions should stay in canonical order");
}

#[test]
fn simultaneous_explosions_produce_deterministic_final_state() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 0, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);

    let result = apply_ok(&state, 0, mv(1, 1));

    assert_eq!(result.timeline.steps[1].explosions.len(), 2);
    assert_cell(&result.final_state, 1, 1, Some(0), 2);
    // Each simultaneous explosion should contribute exactly one orb to the center.
    assert_eq!(
        result.timeline.steps[1]
            .transfers
            .iter()
            .filter(|t| t.to == idx(&result.final_state, 1, 1) as u32)
            .count(),
        2
    );
    assert_board_stable(&result.final_state);
}

#[test]
fn identical_state_and_move_produce_identical_timelines() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    set_cell_to_critical_minus_one(&mut state, 1, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 0, 1, 0);
    set_cell_to_critical_minus_one(&mut state, 1, 0, 0);
    set_cell_to_critical_minus_one(&mut state, 1, 2, 0);

    let a = apply_ok(&state, 0, mv(1, 1));
    let b = apply_ok(&state, 0, mv(1, 1));

    assert_eq!(
        serde_json::to_string(&a.timeline).unwrap(),
        serde_json::to_string(&b.timeline).unwrap()
    );
}
