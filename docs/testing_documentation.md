# Testing Documentation

This document describes how the Connect Four AI project was tested.  
It covers unit tests, integration tests, performance experiments, and how these tests can be repeated.

---

## 1. Unit Testing

### 1.1 Tools and coverage

Unit tests are written using Rust’s built‑in test framework (`cargo test`).  
Code coverage was measured with [`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).

At the time of writing:

- Core engine (board logic + outcome) and AI search crate (minimax + alpha–beta + iterative deepening) are well covered.
- TUI code is intentionally not unit tested (course recommendation: focus on core logic, TUI is secondary).

A full coverage percentage (from `cargo llvm-cov`) will be included in the final report; here we focus on **what** was tested and **how**.

---

### 1.2 Engine unit tests (ConnectFourState)

We have unit tests for:

#### a) Win detection (`outcome`)

Examples:

- **Horizontal win**:

    - Set up board so that Max (`X`) has four in a row on the bottom row.
    - Assert `outcome() == Win(MinMaxPlayer::Max)`

- **No win → None**:

    - Board with a few scattered pieces, no 4‑in‑a‑row.
    - Assert `outcome() == None`.

- **Draw detection**:

    - Fill the board completely, ensure no player has 4 in a row.
    - Assert `outcome() == Draw`.

These tests ensure the bitboard win detection and draw logic behave correctly. There are other unit tests as well than
examples above, but these unit tests essentially all test the outcome handling.

#### b) Window enumeration and counting

The heuristic uses 4‑cell “windows” (rows, columns, diagonals). We test:

- On an **empty board**:
    - `collect_four_in_rows()` (test‑only helper) returns exactly 69 windows (24 horizontal, 21 vertical, 12+12 diagonals).
    - Every window `(max, min, empty)` = `(0, 0, 4)`.

- On a board with a known pattern (e.g., 4 Max pieces on the bottom row):
    - Exactly one horizontal window has counts `(4,0,0)`.
    - Some vertical/diagonal windows have `(1,0,3)` as expected.
    - We check that these patterns appear in the returned vector.

This confirms that window enumeration and counting are correct and stable.

#### c) Invariants

The project contains few invariant tests. We test the following invariants:

- **Piece count invariant**:

  - For a few mid‑game positions, test that piece counts for Max and for Min player never differ more
  than by one

- **Terminal vs non-terminal state and legal moves**:

  - If terminal state, then `outcome().is_some()`.
  - If non-terminal state, then `outcome().is_none()`, then there must be at least one legal move.

- **Overlapping pieces**:
  - We play some random mid-game state and verify there's no overlapping pieces.

- **Heights match the internal bitboard state**:
  - In game state the heights state match the player's bitboard representation state.

- **Players alternate**:
  - Play a series of moves and verify that turns alternate by player.

These tests protect against structural bugs.

---

### 1.3 AI unit tests (minimax, alpha–beta, iterative deepening)

We unit‑test the AI logic in the generic `ai` crate using:

- A simple **mock GameState** (to test minimax mechanics at depth 0/1/2).

#### a) Depth 0 tests

- At `depth = 0`, `minimax` must return `evaluate(state)` without exploring moves.
- Tests:

    - For a terminal state:
        - `depth = 0` → score = `WIN_SCORE` / `LOSS_SCORE` / `0` (draw).
    - For a non‑terminal state with a known heuristic:
        - `depth = 0` → score = value from `evaluate`.

This verifies the base case logic.

#### b) Depth 1 and 2 tests (mock GameState)

Using a mock game where:

- Child states have known fixed scores (+1, −1, 0),
- We test:

    - At a **Max** node with two children (scores +1, −1), `minimax(depth=1)` chooses the +1 child.
    - At a **Min** node, it chooses the −1 child.
    - At depth 2, Max correctly picks the move that maximizes the *minimum* over Min’s responses.

These tests confirm that:

- Maximizing and minimizing are correctly implemented,
- Alpha–beta pruning does not change the logical result at a given depth.

#### c) Plain minimax vs iterative deepening


We test that, when configured to search to the same depth, the result from iterative deepening (using a depth cap and a generous time limit) matches the result from plain minimax at that depth.


This ensures iterative deepening is “just a wrapper” on top of minimax.

---

### 1.4 Heuristic unit tests

We test the **heuristic evaluation** separately:

- **Single Max piece in center > edge**:

  - Put one Max piece in a central column vs in an edge column (using test helper).
  - Assert: `score_center > score_edge`

- **3‑in‑a‑row > 2‑in‑a‑row > 1‑in‑a‑row**:

  - Construct boards with:
    - One Max 1‑in‑a‑row window,
    - One Max 2‑in‑a‑row window,
    - One Max 3‑in‑a‑row window.
  - Assert scores as follows:

    ```
    score_three > score_two;
    score_two > score_one;
    ```

- **Symmetry**:

  - The same patterns for Min produce negative scores of the same magnitude.

These tests ensure the heuristic follows the intended design.

---

## 2. Integration Testing

Integration tests live mainly in `crates/engine/tests` and use both the **Connect Four engine** and the **AI** together.

### 2.1 AI blocks immediate win

We test that the AI blocks an opponent’s immediate 4‑in‑a‑row threat if possible.

Example:

- Set up a position via normal `with_move` calls so that:

  - Max has three in a row, open on one side,
  - It is Min’s turn (AI),
  - Playing in a certain column would allow Max to win next move,
  - A blocking move exists.

This checks that the AI does **not** choose the move that gives the opponent an immediate win when a safe move exists.

### 2.2 AI takes immediate win

Similarly:

- A position where the AI (Max or Min) can win in one move.
- We expect the AI to choose the winning column rather than any “positional” move.

This confirms the AI prioritizes winning moves correctly.

### 2.3 Iterative deepening integration

We test that iterative deepening runs correctly from an almost‑empty position.

This is a “smoke test” that:

- The full engine + iterative_minimax pipeline works end‑to‑end,
- A non‑trivial game is played,
- No panics occur, and statistics are consistent.

(We intentionally **do not** assert strict timing limits here because coverage/debug builds distort timings.)

---

## 3. Empirical / Performance Tests

Performance is not tested in `cargo test` directly, but with the **benchmark tool** (a separate binary).

### 3.1 Benchmark setup

- We run AI vs AI games with different configurations:
  - v1 heuristic vs v2 heuristic,
  - Plain minimax (fixed depth) vs iterative deepening (time limited).
- For each configuration we collect:
  - Number of games (e.g., 50),
  - Average nodes per move,
  - Average time per move,
  - Win / loss / draw counts.

Example configuration:

- Depth‑limited baseline:

  ```text
  depth = 6
  time_ms = None
  heuristic_min = V1
  heuristic_max = V1
  ```

- Iterative deepening + heuristic:

  ```text
  depth = 9       // high cap, not always reached
  time_ms = 50
  heuristic_min = V2
  heuristic_max = V2
  ```

### 3.2 Benchmark results 

*[to be filled]*

Plan is to present these results in simple tables and possibly bar charts (nodes/time/wins) in the final report.

---

## 4. How to Reproduce the Tests

### 4.1 Unit and integration tests

From the project root:

run all unit + integration tests `cargo test`

with coverage (optional) `cargo llvm-cov --workspace --ignore-filename-regex 'tui|benchmark'`

### 4.2 Benchmark / performance tests

From the project root:

```
# depth-limited baseline
cargo run -p benchmark --release -- \
    --games 50 --depth 6 --time-ms none \
    --heuristic-min v1 --heuristic-max v1 \
    --output baseline_v1.json

# iterative deepening + v2
cargo run -p benchmark --release -- \
    --games 50 --depth 9 --time-ms 50 \
    --heuristic-min v2 --heuristic-max v2 \
    --output iterative_v2_50ms.json
```

These commands:

- Run the benchmark binary,
- Save results as JSON (total games, nodes, time, outcomes),
- Allow easy plotting or further analysis.

---

## 5. Summary of Testing Approach

- **Unit tests** verify:
  - Board logic (win/draw detection, move legalness),
  - Heuristic correctness and symmetry,
  - Minimax / alpha–beta base cases and behavior at small depths.

- **Integration tests** verify:
  - Engine + AI together behave correctly in tactical situations (taking wins, blocking threats),
  - Iterative deepening and time limited search complete successfully and return valid moves.

- **Performance/empirical tests**:
  - Run via a separate benchmark tool,
  - Compare different algorithm configurations (v1 vs v2, fixed depth vs iterative),
  - Provide node/time and outcome statistics.
