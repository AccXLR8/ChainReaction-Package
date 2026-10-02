#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn cli_style_sequence_produces_expected_chain() {
    let mut state = fresh_game(3, 3, &[0, 1]);
    // Sequence mirrors an interactive CLI session that primes two edges before detonating the center.
    let scripted_moves = [
        (0, 1),
        (2, 2),
        (0, 1),
        (2, 1),
        (1, 0),
        (2, 2),
        (1, 0),
        (2, 2),
        (1, 1),
        (2, 2),
        (1, 1),
        (2, 2),
        (1, 1),
        (2, 2),
        (1, 1),
    ];

    let mut last_result = None;
    for (i, &(row, col)) in scripted_moves.iter().enumerate() {
        let player = state.current_player();
        last_result = Some(apply_ok(&state, player, mv(row, col)));
        state = last_result.as_ref().unwrap().final_state.clone();
        // Break if game already won to keep sequence deterministic.
        if matches!(state.status, engine::GameStatus::Won { .. }) {
            break;
        }
        // After final player-0 move, we intentionally stop without giving player 1 a response.
        if i == scripted_moves.len() - 1 {
            break;
        }
    }

    let result = last_result.expect("sequence should produce a move result");
    assert_eq!(
        result.timeline.steps.len(),
        2,
        "expected chain reaction with two waves"
    );
    assert_eq!(result.timeline.steps[0].explosions.len(), 1);
    assert_eq!(result.timeline.steps[1].explosions.len(), 2);
    assert_cell(&result.final_state, 0, 1, Some(0), 3);
    assert_cell(&result.final_state, 1, 0, Some(0), 3);
    assert_cell(&result.final_state, 1, 1, None, 0);
    assert_board_stable(&result.final_state);
}
