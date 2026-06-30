# Implementation Document

## Project Overview

The goal of the project is to implement an AI for the Connect Four game using the Minimax algorithm with alpha-beta pruning. 
Secondary goal is to investigate how optimization techniques affect search efficiency and achievable search depth.

The baseline implementation consists of a depth-limited Minimax search (with optional alpha-beta pruning). Additional optimizations include:

* Iterative deepening (with time limitation)
* Move ordering
* Transposition tables
* Principal Variation Search
* Heuristic evaluation functions

The implementation is written in Rust language.

---

# Project Structure

The project is a so-called Workspace project. This in Rust ecosystem means a collection of packages, called workspace
members. The workspace is divided into multiple crates. A crate is the smallest compilation unit in Rust.
A crate can be compiled into a binary or into a library.

### Crates in the project
* AI Engine
* Connect Four Engine
* TUI (Text based UI)
* Benchmark tool

## AI Engine

The AI engine is implemented as a generic search framework using Rust traits.

The core abstractions are:

* `GameState`
* `Move`
* `Player`

This allows the algorithm implementation to remain independent of any specific game implementation. Currently only Minimax
has been implemented. Optional goal: Monte Carlo Tree Search (MCTS)

The baseline search algorithm consists of:

1. Move generation
2. Recursive Minimax search
3. Alpha-beta pruning (toggleable)
4. Static evaluation at the search horizon (heuristic)


## Connect Four Game Engine

The game engine is responsible for:

* Representing the Connect Four game state
* Generating legal moves
* Applying moves
* Detecting terminal states (win, loss, draw)
* Evaluating board positions

The board representation is based on bitboards. Each player's pieces are stored in a separate 64-bit integer. 
Win detection is implemented using bitwise operations and directional shifts.

### Transposition Table

Previously evaluated positions are stored in a transposition table implemented using Rust's `HashMap`.

The table maps game states to the best move found from that state. Cached moves are reused to improve move ordering in future searches.
A new table is created for each AI move (each root search) and cleared when the human or the other AI has made a move.

### Iterative Deepening

Iterative deepening performs multiple searches with increasing depth limits. It uses a time limit and configured max depth to stop advancing in the depth and returns the best move so far.

The transposition table is shared between iterations, allowing information discovered in shallow searches to improve move ordering in deeper searches.


## Text based UI (TUI)

A separate UI application was implemented to allow interactive play from console for the Connect Four Game Engine. It allows
two players, either human or AI to play against each other. It uses ascii characters to output the game board state
and uses keyboard inputs for Human to interact with the board.

Based on the course recommendation, the UI isn't unit tested, nor necessarily on the same quality level as the rest of the implementation.
This is purely based on the time and priority spent with the project. 

## Benchmark Tool

A separate benchmark application was implemented for performance evaluation.

The benchmark tool allows:

* Running multiple games automatically
* Measuring search times
* Comparing algorithm variants
* Collecting win, loss and draw statistics

---

# Algorithms

## Minimax

The Minimax algorithm assumes optimal play from both players.

At each node:

* The maximizing player attempts to maximize the evaluation score.
* The minimizing player attempts to minimize the evaluation score.

The search continues until:

* A terminal game state is reached, or the maximum search depth is reached.
* Alternatively, when using time limited search which uses iterative deepening: when time limit has exceeded.

## Alpha-Beta Pruning

Alpha-beta pruning eliminates branches that cannot affect the final decision.

This reduces the number of explored nodes while preserving the correctness of the Minimax result.

## Move Ordering

Legal moves are explored in an order that prioritizes moves near the center column.

In Connect Four, central moves are often stronger than edge moves and therefore more likely to produce alpha-beta cutoffs.

Additionally, moves retrieved from the transposition table are searched first when available.

## Principal Variation Search (PVS)

On top of alpha–beta, the search uses Principal Variation Search (also known as Null or Zero Window Search)
to reduce work on "non‑principal" moves.

At a node where the remaining search depth is sufficiently large (in this
implementation, at least 4 plies), the children are searched as follows:

* The **first child** (the current best candidate move) is searched with a
  full alpha–beta window `[alpha, beta]`.
* The **remaining children** are first searched with a **null window**:
  a very narrow interval just above or below the current bound. This is a cheap
  test to see whether the move can beat the current best move.
* Only if this null-window search indicates that the move might be better than
  the current best (i.e. the score falls “between” alpha and beta) is the move
  re‑searched with the full window `[alpha, beta]`.

At shallower remaining depths (less than 4 plies) PVS is not used. This avoids the overhead of null-window
re‑searches in regions of the tree where move ordering is not yet reliable and
the potential pruning benefit is small.

PVS does not change the result of the search (score or best move); it is purely
an optimization on top of alpha–beta. Unit tests and fixed‑depth benchmarks
were used to verify that the PVS variant returns the same scores and best moves
as the plain iterative deepening Minimax.

## Heuristic Evaluation

When the search depth limit is reached before reaching a terminal state, a heuristic evaluation function is used.

The heuristic evaluates board positions based on features such as:

* Potential winning lines
* Connected pieces
* Board control

---

# Time and Space Complexity

## Minimax

Without pruning, Minimax has time complexity

O(b^m)

where:

* b = branching factor
* m = search depth

The space complexity is

O(m)

because the search is depth-first and stores only the current recursion path.

## Alpha-Beta Pruning

Worst-case complexity remains

O(b^m)

However, effective move ordering can significantly reduce the number of explored nodes.

Effective alpha-beta pruning gets closer to 

O(b^(m/2))

which effectively doubles the achievable search depth for the same amount of computation.

## Transposition Table

Lookup and insertion operations have expected complexity

O(1)

using a hash table.

Memory usage grows approximately linearly with the number of cached positions.

## Iterative Deepening

Iterative deepening performs multiple searches.

Complexity remains similar to Minimax because most nodes are explored during the deepest search iteration.

In practice, iterative deepening often improves overall performance due to improved move ordering.

---

# Performance Evaluation

Performance was evaluated using the benchmark application by running
self‑play games from the initial position under different search settings
(depth limits, optional time limits).

The final search configuration (iterative deepening + alpha–beta + transposition
table + PVS) was measured in terms of:

* Average search time per move
* Average number of explored nodes per move
* Effective search depth reached under a given time budget
* Win, loss and draw statistics in self‑play

---

# Limitations and Possible Improvements

## Current Limitations

The implementation uses a depth-limited search or time-limited search, and therefore cannot guarantee optimal play in all positions.

The heuristic evaluation function is relatively simple and may not accurately evaluate all strategic situations.

The transposition table currently stores only the best move and does not store complete search results.
Cache entries are used only for move ordering; scores are always recomputed at each depth.


## Possible Improvements

Potential future improvements can include features and ideas like:

* More advanced heuristic evaluation functions
* Enhanced transposition table entries
* Parallel search
* Optional: Comparison to Monte Carlo Tree Search (MCTS) implementation 

---

# Use of Large Language Models

Large language models were used during the development of the project.

ChatGPT (OpenAI GPT-5.5) was used for:

* Discussing possible unit test cases
* Reviewing documentation and language used (style and grammar)
* Interpretation of results vs implemented improvements

---

# References used

1. Russell, S., Norvig, P. *Artificial Intelligence: A Modern Approach*.
2. Alpha-Beta Pruning, Wikipedia and Geeks for Geeks
3. Connect Four Bitboard Techniques (Google)
4. Rust Programming Language Documentation.
