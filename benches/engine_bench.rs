//! Criterion benchmarks exercising the hot simulation paths under varied scenarios.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use engine::{apply_move, new_game, replay, GameState, Move};

/// Measures a tiny board with minimal reactions.
fn bench_small_board(c: &mut Criterion) {
    c.bench_function("small_board_single_move", |b| {
        b.iter_batched(
            || new_game(3, 3, vec![0, 1]).unwrap(),
            |state| {
                let _ = apply_move(&state, 0, Move::new(0, 0)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Measures mid-sized boards with mid-grid placements.
fn bench_medium_board(c: &mut Criterion) {
    c.bench_function("medium_board_move", |b| {
        b.iter_batched(
            || new_game(8, 8, vec![0, 1]).unwrap(),
            |state| {
                let _ = apply_move(&state, 0, Move::new(4, 4)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Measures large boards to stress indexing cost.
fn bench_large_board(c: &mut Criterion) {
    c.bench_function("large_board_move", |b| {
        b.iter_batched(
            || new_game(12, 12, vec![0, 1]).unwrap(),
            |state| {
                let _ = apply_move(&state, 0, Move::new(6, 6)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Times the cost of a single-corner explosion.
fn bench_single_explosion(c: &mut Criterion) {
    c.bench_function("single_corner_explosion", |b| {
        b.iter_batched(
            || corner_ready_state(),
            |state| {
                let _ = apply_move(&state, 0, Move::new(0, 0)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmarks a two-wave chain reaction.
fn bench_short_chain(c: &mut Criterion) {
    c.bench_function("short_chain_two_waves", |b| {
        b.iter_batched(
            || multi_wave_state(3),
            |state| {
                let _ = apply_move(&state, 0, Move::new(1, 1)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmarks a larger wave set triggered from the center.
fn bench_medium_chain(c: &mut Criterion) {
    c.bench_function("medium_chain_three_waves", |b| {
        b.iter_batched(
            || multi_wave_state(5),
            |state| {
                let row = state.board.height() / 2;
                let col = state.board.width() / 2;
                let _ = apply_move(&state, 0, Move::new(row, col)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Explores a near-cross board that sends large waves.
fn bench_large_chain(c: &mut Criterion) {
    c.bench_function("large_chain_many_waves", |b| {
        b.iter_batched(
            || near_critical_board(8),
            |state| {
                let mid = state.board.width() / 2;
                let _ = apply_move(&state, 0, Move::new(mid, mid)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Tests a worst-case dense setup where every cell is primed.
fn bench_worst_case_chain(c: &mut Criterion) {
    c.bench_function("worst_case_chain", |b| {
        b.iter_batched(
            || fully_charged_board(10),
            |state| {
                let _ = apply_move(&state, 0, Move::new(5, 5)).unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

/// Applies a pseudo-random sequence of legal moves on a large board.
fn bench_random_moves(c: &mut Criterion) {
    c.bench_function("random_legal_moves", |b| {
        let moves = generate_move_sequence(64, 12, 12);
        b.iter_batched(
            || new_game(12, 12, vec![0, 1]).unwrap(),
            |mut state| {
                for mv in &moves {
                    let player = state.current_player();
                    if let Ok(result) = apply_move(&state, player, *mv) {
                        state = result.final_state;
                    } else {
                        break;
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

/// Simulates an entire game, stopping when someone wins.
fn bench_full_game(c: &mut Criterion) {
    c.bench_function("full_game_simulation", |b| {
        let moves = generate_move_sequence(200, 6, 6);
        b.iter_batched(
            || new_game(6, 6, vec![0, 1]).unwrap(),
            |mut state| {
                for mv in &moves {
                    let player = state.current_player();
                    if let Ok(result) = apply_move(&state, player, *mv) {
                        state = result.final_state;
                    }
                    if matches!(state.status, engine::GameStatus::Won { .. }) {
                        break;
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

/// Measures throughput of the replay helper on a long log.
fn bench_replay(c: &mut Criterion) {
    c.bench_function("replay_long_game", |b| {
        let moves = generate_move_sequence(256, 5, 5);
        let initial = new_game(5, 5, vec![0, 1]).unwrap();
        b.iter(|| {
            let _ = replay(&initial, &moves);
        });
    });
}

/// Prepares a board where the top-left corner will explode next move.
fn corner_ready_state() -> GameState {
    let mut state = new_game(3, 3, vec![0, 1]).unwrap();
    let idx = state.board.index(0, 0).unwrap();
    let cell = state.board.cell_mut(idx);
    cell.owner = Some(0);
    cell.orb_count = 1;
    state
}

/// Seeds a board so the center triggers multi-wave reactions.
fn multi_wave_state(size: u16) -> GameState {
    let mut state = new_game(size, size, vec![0, 1]).unwrap();
    let row = size / 2;
    let col = size / 2;
    let center = state.board.index(row, col).unwrap();
    {
        let board = &mut state.board;
        let center_cell = board.cell_mut(center);
        center_cell.owner = Some(0);
        center_cell.orb_count = board.critical_mass_index(center) - 1;
        for neighbor in board.neighbors(center) {
            let cell = board.cell_mut(neighbor);
            cell.owner = Some(0);
            cell.orb_count = board.critical_mass_index(neighbor) - 1;
        }
    }
    state
}

/// Charges a plus-shaped cross to just below critical mass.
fn near_critical_board(size: u16) -> GameState {
    let mut state = new_game(size, size, vec![0, 1]).unwrap();
    let mid = size / 2;
    for row in 0..size {
        for col in 0..size {
            if row == mid || col == mid {
                let idx = state.board.index(row, col).unwrap();
                let cell = state.board.cell_mut(idx);
                cell.owner = Some(0);
                let critical = state.board.critical_mass(row, col);
                cell.orb_count = critical.saturating_sub(1);
            }
        }
    }
    state
}

/// Sets every cell to one orb shy of exploding for a worst-case chain.
fn fully_charged_board(size: u16) -> GameState {
    let mut state = new_game(size, size, vec![0, 1]).unwrap();
    for row in 0..size {
        for col in 0..size {
            let idx = state.board.index(row, col).unwrap();
            let cell = state.board.cell_mut(idx);
            cell.owner = Some(0);
            cell.orb_count = state.board.critical_mass(row, col).saturating_sub(1);
        }
    }
    state
}

/// Generates a deterministic pseudo-random move stream via an xorshift64* variant.
fn generate_move_sequence(count: usize, width: u16, height: u16) -> Vec<Move> {
    let mut moves = Vec::with_capacity(count);
    let mut seed = 0xdead_beefu64;
    for _ in 0..count {
        seed ^= seed >> 12;
        seed ^= seed << 25;
        seed ^= seed >> 27;
        seed = seed.wrapping_mul(2685821657736338717);
        let row = (seed as u16) % height;
        let col = ((seed >> 16) as u16) % width;
        moves.push(Move::new(row, col));
    }
    moves
}

criterion_group!(
    benches,
    bench_small_board,
    bench_medium_board,
    bench_large_board,
    bench_single_explosion,
    bench_short_chain,
    bench_medium_chain,
    bench_large_chain,
    bench_worst_case_chain,
    bench_random_moves,
    bench_full_game,
    bench_replay,
);
criterion_main!(benches);
