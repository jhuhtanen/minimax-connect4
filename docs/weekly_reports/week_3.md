# Weekly report

## Summary
This week I completed the heuristic implementation and iterative deepening.

### Description
I spent quite a bit of time to unit test the heuristic implementation that now is based
on two principles: 1. Prefer the center columns and have weights that in scoring prefer
these center columns opposed to the edge columns. 2. Look for all possible 4 in row windows
of slots and inspect and value them so that 3 in row (one empty on either side) is more valuable
than 2 in rows (one empty on either side) and lastly 1 in row. This should be a simple
enough for "threat" evaluation. This seems to be working, but testing it on the benchmark or tui
is difficult to make very big differentiation to non-heuristic version.

I also spent some time to keep my tui and benchmarks up to date of the changes in
the AI crate. This clearly adds to time spent with the project. Even if it's not necessarily
a direct goal.

I managed to also implement the first pass on the iterative deepening and constraining the
move time. I added the transposition tables (caching of best moves) as well. These are not as
well unit tested as the rest of the codebase. This seems to be working as well, but
maybe I will run more experiments next week.

For the minimum required version I still need to add the move ordering and victory condition
checking to use only last move. I assume these aren't very big nor complex changes. Having said this
for the victory condition checking I need to pass the move which might cause
some refactoring needs.

## Time spent
24.5  : 4 hours  
26.5  : 3 hours  
27.5  : 2 hours  
30.5  : 5 hours  

## Progression of project
- Minimax heuristics implementation. 
- Parametrisation: Time based constraining.
- Iterative deepening: First pass completed.
- TUI and benchmarks: Updated accordingly.

## Learnings
- Comparing different versions of minimax and understanding the differences
is difficult. It's sometimes hard to say why some stats change. My understanding
is that limiting the depth makes the algorithm "not to see" beyond the state and
making an educated guess with the heuristic. And that still leads to some poor
choices. Ideally I would like to see more draws in stats, but I suppose that
require pretty high depth and compute times.

## Planned next
- Move ordering. Try middle ones first, move towards the perimeter of the board.
- Win check improvements: Check only rows that can contain the previous move, it must be part of the win row.
- Improve unit testing. There's new functionality that isn't properly tested.

## Difficulties or unclear topics
- None


## Questions or feedback to lecturers and assistants
- I wonder if it would be possible to check the iterative deepening I've implemented
that it actually is based on the same idea that course requires. AI crate has now two versions
of the minimax algorithm, one with iterative deepening and caching, second one without.
- With the current state of the minimax and if I implement the improvements listed above,
can I move to study MCTS and does it make sense?