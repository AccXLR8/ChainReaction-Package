#[path = "common/mod.rs"]
mod common;

use common::*;
use engine::replay;

#[test]
fn replay_matches_live_execution() {
    let initial = fresh_game(4, 4, &[0, 1]);
    let moves = [mv(0, 0), mv(1, 1), mv(0, 0), mv(2, 2), mv(0, 0)];

    let mut state = initial.clone();
    for mv in &moves {
        let player = state.current_player();
        state = apply_ok(&state, player, *mv).final_state;
    }

    let replayed = replay(&initial, &moves).expect("replay succeeds");
    assert_eq!(snapshot(&replayed), snapshot(&state));
}
