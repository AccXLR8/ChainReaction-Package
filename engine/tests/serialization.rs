#[path = "common/mod.rs"]
mod common;

use common::*;
use engine::{apply_move, GameState, MoveResult};

#[test]
fn game_state_round_trips_through_json() {
    let mut state = fresh_game(5, 5, &[0, 1]);
    set_cell(&mut state, 0, 0, 0, 1);
    set_cell(&mut state, 2, 2, 1, 3);

    let serialized = serde_json::to_string(&state).unwrap();
    let restored: GameState = serde_json::from_str(&serialized).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&state));
}

#[test]
fn move_result_round_trips_with_timeline() {
    let state = fresh_game(3, 3, &[0, 1]);
    let result = apply_move(&state, 0, mv(0, 0)).unwrap();
    let json = serde_json::to_string(&result).unwrap();
    let restored: MoveResult = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), json);
}
