# Requirement specification

This document describes the requirements for "TKT20010 Aineopintojen harjoitustyö: Tietorakenteet ja algoritmit" (Algorithms and AI) course. The course is part of TKT (Tietojenkäsittelytieteen kandidaatti) curriculum.

## Programming language

The project is written in **Rust**. It uses language features up to **1.85.0** (rustc 1.85.0). There might be some helper scripts to run / execute project code that can be in other scripting languages like bash.

## Implemented algorithm

The project implements a Minimax solver for Connect 4 game. The baseline for the Minimax will be a default minimax implementation with alpha-beta pruning. Minimax recursively traverses an implicit game tree generated from legal game states. The initial prototype implementation will use Tic Tac Toe. This is only for prototyping / testing purposes. The reason is that in Tic Tac Toe game the board size is fixed (3 by 3 cells) allowing fixed recursion depth and relatively small branching factor. The project uses compact board representations for Connect 4 states, recursive search trees for Minimax traversal and hash-based transposition tables for caching previously evaluated positions.

Optional: If there's time and the author feels confident, an optional Monte Carlo Tree Search implementation may be added. Due to the differences in the algorithms nature, the comparison most likely happen how solvers play against each other with a fixed amount of computing time allowed per algorithm.

## Problem statement

The problem solved in this project is how to efficiently search large Connect 4 game trees in order to create a competitive AI player. Exhaustive search is expensive for Connect 4 game therefore the project investigates how Minimax with alpha-beta pruning and related optimisations can improve search efficiency and playing strength.

## Project core

The core of the project is the implementation and optimisation of Minimax search with alpha-beta pruning for Connect 4. The main focus is improving search efficiency by using optimisations such as move ordering, iterative deepening, heuristic evaluation functions and transposition tables. The Connect 4 game state and functionality is also part of the project core. Text based UI will be added for allowing visualisation of the game state.

## Algorithm inputs

The Minimax algorithm will at the baseline get as inputs a Tic Tac Toe board state containing the cell occupancy data, the current player to move, alpha-beta parameters. When baseline has been validated and tested, new baseline with Connect 4 will be created. This baseline uses Connect 4 board state, the current player to move, alpha-beta parameters. With improved / optimised versions the inputs should stay the same as the optimisations are implemented inside the algorithm.

Optional: The MCTS algorithm inputs will be a Connect 4 board state. If comparison will be done, additional inputs might be needed like maximum time for the solver to compute one move.

## Time and space complexity

Minimax algorithm has time complexity of O(b^m) where b is branching factor and m is maximum depth of the search tree. The space complexity is similar to any DFS (depth first search) which is O(m).

Optional: The time complexity of MCTS depends on the number of iterations. Space complexity grows approximately linearly with the number of explored states stored in the search tree.


## Source material

- Russell & Norvig (Artificial Intelligence: A Modern Approach)
- https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
- https://inst.eecs.berkeley.edu/~cs188/textbook/games/minimax.html
- https://www.geeksforgeeks.org/artificial-intelligence/mini-max-algorithm-in-artificial-intelligence/
- https://stanford.edu/~rezab/classes/cme323/S15/projects/montecarlo_search_tree_report.pdf
- https://www.geeksforgeeks.org/machine-learning/monte-carlo-tree-search-mcts-in-machine-learning/

## Author's peer review languages

The author can review other projects written in: `Rust`, `C/C++`, `C#`, `Java`, `JavaScript / TypeScript` and `Python`

## Language

Project language is **English**. All the documentation and comments in the code is written in English. Author can work with professional level in English or Finnish.

