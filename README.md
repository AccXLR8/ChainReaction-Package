# Chain Reaction Engine

Deterministic, production-quality core rules engine for Chain Reaction–style games. The library owns the authoritative game state and exposes a pure Rust API that can be embedded in services, AI simulations, CLIs, or future Python/WASM bindings. There is intentionally no networking, rendering, or persistence surface area — only game logic.

## Features

- Deterministic simulation: identical inputs (initial state + move sequence) always produce identical outputs.
- Complete rule implementation covering board geometry, critical masses, ownership, chain reactions, simultaneous waves, elimination, and victory.
- Explicit reaction timeline suitable for animation layers (`placement`, ordered `ReactionStep`s with explosions + transfers).
- Legal move validation and structured error reporting with `IllegalMove`.
- Replay helpers to rebuild game states from logged moves.
- Serde serialization for every public type.
- Property tests and integration tests that cover geometry, placements, reactions, simultaneity, elimination, invalid moves, replay, and determinism.
- Criterion benchmarks that measure hot paths from single moves through worst-case chains and long game replays.
- CLI development harness with ASCII board rendering for manual play/testing.

## Workspace Layout

```
chain-reaction-engine/
├── Cargo.toml          # Workspace + shared dependencies
├── README.md
├── engine/             # Core library crate
│   ├── src/
│   │   ├── board.rs
│   │   ├── cell.rs
│   │   ├── engine.rs
│   │   ├── error.rs
│   │   ├── game_state.rs
│   │   ├── mv.rs
│   │   ├── player.rs
│   │   ├── reaction.rs
│   │   ├── result.rs
│   │   └── lib.rs
│   └── benches/engine_bench.rs (registered at workspace root)
├── cli/                # Optional ASCII harness using the library
│   └── src/main.rs
├── benches/
│   └── engine_bench.rs # Criterion suites (11 coverage areas)
└── tests/
    ├── board_tests.rs
    ├── chain_tests.rs
    ├── determinism_tests.rs
    ├── elimination_tests.rs
    ├── explosion_tests.rs
    ├── move_tests.rs
    ├── property_tests.rs
    ├── replay_tests.rs
    └── simultaneous_tests.rs
```

## Quick Start

```bash
# run all tests + property suites
cargo test

# run the CLI harness (default 5x5 board with players 0 and 1)
cargo run -p cli -- --width 6 --height 4 --players 0,1

# run benchmarks (build only without executing)
cargo bench --no-run
```

### Library API Overview

```rust
use engine::{new_game, validate_move, apply_move, replay, Move};

let mut state = new_game(8, 8, vec![0, 1]).unwrap();
validate_move(&state, state.current_player(), Move::new(3, 3)).unwrap();
let result = apply_move(&state, state.current_player(), Move::new(3, 3)).unwrap();
println!("reaction steps: {}", result.timeline.steps.len());
state = result.final_state;

let replayed = replay(&state, &[Move::new(0, 0)]).unwrap();
assert_eq!(replayed.board, state.board);
```

## Key Architectural Decisions

- **Flat board representation** (`Vec<Cell>`) indexed by `row * width + col` to avoid nested allocations and to keep neighbor lookups cache-friendly.
- **Deterministic wave simulation**: each reaction wave collects all unstable cells, sorts them by cell index, resolves all explosions, and builds `ReactionStep` and `OrbTransfer` entries while preparing the next wave. Orthogonal neighbor order is fixed as UP → RIGHT → DOWN → LEFT across the entire engine.
- **Player lifecycle tracking**: players are not eliminated until they have moved at least once and subsequently own zero orbs. After every move (and full stabilization) elimination and win conditions are recomputed before advancing the turn to the next active player.
- **Serialization boundary**: all externally useful structs derive `Serialize` / `Deserialize` to support future bindings (PyO3, WASM, CLI, AI). The CLI and future adapters should treat the library as the single source of truth.
- **Replay + determinism**: `replay` applies a move list to any starting state, and determinism tests serialize entire states to prove byte-for-byte equivalence across runs.

## Assumptions and Clarifications

- The current rule set uses a standard 4-neighbor grid (UP, RIGHT, DOWN, LEFT). Critical mass equals the neighbor count: corners → 2, edges → 3, interior → 4.
- Ownership of a cell changes immediately when an orb is transferred into it, even if another transfer happens later in the same wave. Transfer order follows the global direction order, so simultaneous explosions remain deterministic.
- Cells lose exactly their critical mass when exploding and become empty unless re-filled by another transfer in the same wave.
- A player becomes eligible for elimination only after making a move. Until then, a zero-orb player still receives turns (mirrors classic Chain Reaction rules where early knockouts are impossible).

## Benchmark Coverage

The Criterion suite reports latency distributions for:

1. Small board single moves
2. Medium board moves
3. Large board moves
4. Single-corner explosions
5. Short (2-wave) chains
6. Medium chains
7. Large cross-board chains
8. Worst-case near-critical boards
9. Deterministic pseudo-random legal move sequences
10. Full game simulations (stop on victory)
11. Long-game replay throughput

Use `cargo bench` (or `cargo bench --no-run` to simply build) to inspect performance on your hardware.

## Extending the Engine

- **Python bindings**: wrap the existing API with PyO3 in a separate crate (e.g., `bindings/python`). The core engine has no Python-specific dependencies.
- **WebAssembly**: compile the library with `wasm32-unknown-unknown` and export FFI-compatible wrappers — the code avoids OS-only APIs.
- **AI / simulations**: drive the engine via `apply_move` + `MoveResult` timelines. Deterministic results make large-scale Monte Carlo rollouts reproducible.

Contributions welcome! Please accompany behavior changes with tests and keep determinism front-and-center.
