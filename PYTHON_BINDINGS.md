# Python Bindings for the Chain Reaction Engine

This document explains how to build and use the PyO3-based bindings that expose the Rust Chain Reaction engine to Python. It is the authoritative integration guide for any Python consumer.

## 1. Build & Install the Python library

The bindings live in the `python-bindings` crate and build into a `chain_reaction` Python extension module using [PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs/).

1. Install maturin (once per environment):
   ```bash
   pip install maturin
   ```
2. Build and install the module into your current virtual environment:
   ```bash
   cd python-bindings
   maturin develop  # or: maturin develop --release
   ```
   This compiles the Rust code and installs an importable `chain_reaction` package.
3. To build a distributable wheel:
   ```bash
   maturin build --release
   ```
   The crate targets Python's stable ABI (`abi3`), so a single wheel covers CPython ≥3.9.

## 2. Importing the module

```python
import chain_reaction as cr
```

The module exports both free functions (`new_game`, `apply_move`, …) and Python classes mirroring key Rust types.

## 3. Exposed classes & types

### `Move`
- Constructor: `Move(row: int, col: int)`.
- Properties: `row`, `col`.
- Helpers: `as_tuple()` returns `(row, col)`; `repr` shows `Move(row=?, col=?)`.

### `GameState`
Represents a complete snapshot of the board.
- Read-only properties: `width`, `height`, `current_player`, `turn_number`, `status` (`GameStatus`).
- Methods:
  - `clone_state()` → deep copy.
  - `living_players()` → list of surviving player IDs.
  - `players()` → list of dicts `{"id": int, "eliminated": bool, "has_moved": bool}` in turn order.
  - `board_snapshot()` → 2‑D list shaped `[row][col]`; each entry is `None` for empty cells or `{"owner": int, "orb_count": int}`.
  - `to_json()` / `GameState.from_json(str)` for serialization (lossless round‑trip via the engine’s Serde format).
- `__repr__` summarizes the board size, turn number, and status.

### `GameStatus`
- Methods: `kind()` (`"active"` or `"won"`), `winner()` (optional player ID), `eliminated()` (list of eliminated players).

### `MoveResult`
Result from `apply_move`.
- Properties: `final_state` (`GameState`), `timeline` (`ReactionTimeline`), `outcome` (`GameResult`).
- Methods: `to_json()` for archival, `__repr__` summarizing status.

### `ReactionTimeline`
- Properties: `placement` (`OrbPlacement`), `steps` (`List[ReactionStep]`).
- Method: `is_empty()` indicates whether any explosions occurred.

### `ReactionStep`
- Properties: `explosions` (`List[Explosion]`), `transfers` (`List[OrbTransfer]`).

### `OrbPlacement`, `Explosion`, `OrbTransfer`
- Simple value objects exposing `cell` indices and `player` ownership; `OrbTransfer` additionally has `from_cell` / `to_cell`.

### `GameResult`
- Properties: `status` (`GameStatus`), `winner`, `losers`.
- Method: `status_kind()` convenience alias for `status.kind()`.

## 4. Exposed functions

| Function | Arguments | Returns | Notes |
| --- | --- | --- | --- |
| `new_game(width, height, players)` | `int`, `int`, `Iterable[int]` of player IDs | `GameState` | Validates board & players. Errors raise `InvalidBoardError` / `InvalidStateError`. |
| `validate_move(state, player, move)` | `GameState`, `int`, `Move` | `None` | Raises `IllegalMoveError` if the move cannot be played. |
| `apply_move(state, player, move)` | `GameState`, `int`, `Move` | `MoveResult` | Clones `state`, applies the move, returns the next state plus metadata. Raises `EngineError` subclasses. |
| `replay(initial_state, moves)` | `GameState`, `Sequence[Move]` | `GameState` | Sequentially applies moves by following the current-player rotation. Raises `ReplayError` with step information. |

## 5. Return values and structure

- `GameState` objects are immutable snapshots; the engine clones internally, so the input state you pass into `apply_move` or `replay` is never mutated.
- `MoveResult.final_state` is the state to feed into the next turn.
- `MoveResult.timeline` records the full reaction chain starting from `timeline.placement` through an ordered list of `ReactionStep`s.
- `MoveResult.outcome` reports whether the game remains active or which player won, including the eliminated order.

## 6. Passing `GameState` between calls

Use the same `GameState` instance you received from `new_game`, `apply_move`, or `replay`. Because the engine clones its input internally, you may safely reuse a state for validation without worrying about accidental mutation. To serialize a state (for storage or IPC) call `state.to_json()` and later reconstruct it with `GameState.from_json(json_blob)`.

## 7. Constructing moves

```python
move = cr.Move(row=2, col=3)
move.row  # 2
move.col  # 3
row, col = move.as_tuple()
```

Passing plain tuples is not supported; construct `Move` objects explicitly for clarity and type safety.

## 8. Working with `MoveResult`, timelines, and outcomes

```python
result = cr.apply_move(state, player=0, move=move)
next_state = result.final_state
timeline = result.timeline
print(timeline.placement.cell, timeline.placement.player)
for step_no, step in enumerate(timeline.steps, 1):
    print("wave", step_no)
    for explosion in step.explosions:
        print("  explosion", explosion.cell, "owner", explosion.player)
    for transfer in step.transfers:
        print("  transfer", transfer.from_cell, "→", transfer.to_cell)
outcome = result.outcome
print(outcome.status.kind(), outcome.winner, outcome.losers)
```

## 9. Engine errors in Python

The module exposes structured exception types:

- `chain_reaction.EngineError` (base class)
  - `IllegalMoveError`
  - `InvalidBoardError`
  - `InvalidStateError`
- `chain_reaction.ReplayError`

Each exception instance attaches a `.detail` dictionary describing the Rust enum variant. Examples:

```python
try:
    cr.validate_move(state, player=1, move=move)
except cr.IllegalMoveError as exc:
    print(exc)            # human-readable message
    print(exc.detail)     # {'kind': 'NotPlayersTurn', 'message': 'not this player's turn'}
```

```python
try:
    cr.replay(state, [cr.Move(0, 0)])
except cr.ReplayError as exc:
    print(exc.detail)
    # {'kind': 'illegal_move', 'step': 0, 'error': {... full IllegalMove detail ...}}
```

You can catch `EngineError` to handle all engine-generated issues together or catch specific subclasses for finer control.

## 10. Usage examples

### Create a new game
```python
import chain_reaction as cr

state = cr.new_game(width=6, height=8, players=[0, 1])
print(state.width, state.height)
print(state.players())
```

### Make and validate a move
```python
move = cr.Move(row=2, col=3)
cr.validate_move(state, player=0, move=move)  # raises on failure
result = cr.apply_move(state, player=0, move=move)
state = result.final_state  # advance to next turn
```

### Validate without applying
```python
try:
    cr.validate_move(state, player=1, move=move)
except cr.IllegalMoveError as exc:
    print("cannot play:", exc.detail)
```

### Replay a transcript
```python
transcript = [
    cr.Move(0, 0),
    cr.Move(0, 1),
    cr.Move(1, 0),
]
replayed = cr.replay(state.clone_state(), transcript)
print(replayed.turn_number)
```

### Inspect results and timelines
```python
result = cr.apply_move(state, player=state.current_player, move=cr.Move(1, 1))
print("status:", result.outcome.status.kind())
print("winner:", result.outcome.winner)
print("living players after move:", result.final_state.living_players())
print("board snapshot:")
for row in result.final_state.board_snapshot():
    print(row)
```

## 11. JSON & serialization

- `GameState.to_json()` / `GameState.from_json()` allow round-tripping states across process boundaries or storing them in test fixtures.
- `MoveResult.to_json()` captures the final state, reaction timeline, and outcome in a single blob (matching the Rust Serde layout). There is no `from_json` helper for `MoveResult`; reconstruct it by parsing JSON yourself or by replaying moves.

## 12. Limitations & differences vs. Rust API

- The Python module mirrors the Rust engine but does not expose low-level board mutation APIs. Use the provided functions to ensure the engine remains authoritative.
- `GameState` objects returned to Python are immutable snapshots; the engine clones internally, so repeated calls may increase memory usage if you retain every state.
- `board_snapshot()` flattens private Rust data (cells) into serializable dictionaries; it is intended for analytics/visualization rather than heavy-duty simulation.
- Moves must be created with `Move(row, col)` or obtained from earlier calls; plain tuples or dictionaries are not implicitly converted.
- Reaction timelines reference cells by their flat indices (matching the Rust engine); convert to `(row, col)` using your own helpers if needed.
- The bindings deliberately avoid FastAPI or additional backends per project requirements.
- No extra gameplay logic has been added—behavior matches the Rust engine exactly.

With these bindings, any Python program can drive deterministic Chain Reaction simulations while accessing the full fidelity (game states, reaction timelines, outcomes, and detailed errors) produced by the Rust core.
