# Chain Reaction Engine Integration Guide

This library is intended to be the authoritative game-rules core for any service that needs to host or analyze a deterministic two-player Chain Reaction match. External applications can embed it directly (Rust), wrap it through FFI (Python, WASM, etc.), or drive it from the included CLI harness for manual testing.

---

## Public Rust API

All exported symbols live in the `engine` crate (`engine/src/lib.rs`). Key types and functions:

| API | Description |
| --- | --- |
| `engine::new_game(width, height, players)` | Creates a fresh `GameState`. `players` must be a `Vec<PlayerId>` (u8), strictly two distinct IDs (e.g., `[0, 1]`). Returns `Result<GameState, EngineError>`. |
| `engine::validate_move(state, player, mv)` | Ensures `player` may legally place an orb at `mv` (row/col). Returns `Result<(), IllegalMove>`. Useful for pre-flight checks without mutating state. |
| `engine::apply_move(state, player, mv)` | Authoritative state transition. Consumes `state`, validates the move, runs the entire deterministic chain reaction, and returns `Result<MoveResult, EngineError>`. The returned `MoveResult` contains the new `GameState`, the full `ReactionTimeline`, and a `GameResult`. |
| `engine::replay(initial_state, moves)` | Re-applies a series of moves to an initial state. Useful for server-side replay validation and offline analysis. Returns `Result<GameState, ReplayError>`. |
| `engine::GameState` | Snapshot of the entire game (board, players, current player, turn number, status). Implements `Serialize`/`Deserialize` via Serde. |
| `engine::Board` | Flat board representation (width, height, cells). Provides helper methods like `index`, `coords`, `cell`, `critical_mass`, etc. |
| `engine::Move` | Simple row/column pair (`Move::new(row, col)`). Also serializable via Serde. |
| `engine::MoveResult` | Returned from `apply_move`: `{ final_state, timeline, outcome }`. |
| `engine::ReactionTimeline` | Ordered list of `ReactionStep`s (waves) for animation/replay: each step has simultaneous `explosions` plus explicit `OrbTransfer`s. |
| `engine::GameResult` | Captures `GameStatus` (`Active` or `Won { winner, eliminated }`), plus convenience fields for winner/loser IDs. |
| `engine::EngineError` / `IllegalMove` / `InvalidState` / `ReplayError` | Structured error enums so callers can pattern-match on exact failure cases. |

### Typical Control Flow

```rust
use engine::{new_game, validate_move, apply_move, Move};

let mut state = new_game(5, 5, vec![0, 1]).unwrap();

// Player 0 attempts a move:
let mv = Move::new(1, 2);
validate_move(&state, 0, mv)?;          // optional pre-check
let result = apply_move(&state, 0, mv)?; // authoritative update

state = result.final_state;             // persist/store new state
if let Some(winner) = result.outcome.winner {
    // handle finished game; timeline is fully populated
}
```

*Determinism guarantee:* `state + player + move` always yields the same `MoveResult`. All collections use canonical ordering (e.g., neighbor order: Up → Right → Down → Left).

---

## CLI Harness (Development/Diagnostics)

The `cli` crate builds a development harness for manual play/inspection.

### Build & Run

```bash
cd chain-reaction-engine
cargo run -p cli -- --width 5 --height 5 --players 0,1
```

CLI arguments (via Clap):

| Flag | Default | Meaning |
|------|---------|---------|
| `--width <u16>`  | `5` | Board width. Must be ≥ 2. |
| `--height <u16>` | `5` | Board height. Must be ≥ 2. |
| `--players <list>` | `0,1` | Comma-separated player IDs (exactly two). |

Once running, the CLI prints the ASCII board, current player, and prompts for moves (`row col`). After each move it shows the reaction summary wave-by-wave using the same event data returned by `ReactionTimeline`.

The CLI is strictly for development/testing—the core engine does not depend on it.

---

## Serialization/Interop

All public structs derive `Serialize`/`Deserialize` (Serde), so they can be:

* Stored/restored as JSON or any Serde-compatible format.
* Exposed through PyO3, WASM bindings, or HTTP APIs.

Example JSON snippet of `MoveResult`:

```json
{
  "final_state": { "board": { ... }, "players": [...], ... },
  "timeline": {
    "placement": { "cell": 12, "player": 0 },
    "steps": [
      {
        "explosions": [{ "cell": 12, "player": 0 }],
        "transfers": [
            { "from": 12, "to": 7, "player": 0 },
            ...
        ]
      }
    ]
  },
  "outcome": {
    "status": { "Won": { "winner": 0, "eliminated": [1] } },
    "winner": 0,
    "losers": [1]
  }
}
```

This stable schema makes it easy for external services to:

1. Persist authoritative states.
2. Broadcast reaction timelines to clients.
3. Reconstruct past games exactly.

---

## Error Handling

All public APIs return structured errors. External apps should match on the variants to decide behavior:

```rust
match apply_move(&state, player, mv) {
    Ok(result) => { ... }
    Err(engine::EngineError::IllegalMove(cause)) => match cause {
        IllegalMove::OutOfBounds => { ... }
        IllegalMove::CellOwnedByOpponent { owner } => { ... }
        IllegalMove::GameAlreadyFinished => { ... }
        _ => { ... }
    },
    Err(other) => { /* invalid state/board/etc. */ }
}
```

---

## Summary Checklist for External Integrations

- **Initialization:** call `new_game(width, height, vec![0,1])`.
- **Move loop:** per turn, use `apply_move` (optionally `validate_move` first).
- **Persistence:** serialize/deserialize `GameState` or entire `MoveResult`.
- **Replay:** use `replay(initial_state, moves)` to verify logs or audit games.
- **Timeline export:** `ReactionTimeline` is ready for animation/playback frontends (wave-by-wave).
- **Determinism:** no RNG or time-based behavior—safe for server authority, AI rollouts, and tournament adjudication.
- **Strict two-player support:** elimination and win detection are built around exactly two active players; the engine halts the moment a post-wave check finds only one survivor.

Use this document as the external-facing reference: pick the API that fits (direct Rust, FFI wrapper, CLI) and drive the deterministic Chain Reaction simulation exactly once per move.
