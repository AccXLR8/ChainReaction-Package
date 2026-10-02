#[path = "common/mod.rs"]
mod common;

use common::*;
use engine::apply_move;
use proptest::prelude::*;

proptest! {
    #[test]
    fn random_sequences_leave_board_stable(moves in prop::collection::vec((0u16..3, 0u16..3), 1..40)) {
        let mut state = fresh_game(3, 3, &[0, 1]);
        for (row, col) in moves {
            let player = state.current_player();
            match apply_move(&state, player, mv(row, col)) {
                Ok(result) => state = result.final_state,
                Err(_) => { /* ignore illegal moves */ }
            }
        }
        assert_board_stable(&state);
    }
}
