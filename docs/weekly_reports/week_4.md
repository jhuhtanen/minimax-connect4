# Weekly report

## Summary
This week I have mostly been preparing for the peer review happening next week. I discussed with the lecturer about
the project state and possible current shortcomings and potential few improvements I could still try.

### Description
I started to complete the implementation documentation and testing documentation. They are not yet complete, but should
give enough understanding and context for the peer review. 

Additionally, I spent some time in refactoring the non-critical components, benchmark and tui to make them more modular
and readable. I added unit tests to the benchmark (partially).

I improved the Connect Four engine crate documentation. I added rust code documentation to the most relevant bits making
peer reviewing easier and crate doc generation to be more complete.

## Time spent
31.5  : 4 hours  
3.6   : 2 hours  
5.6   : 4 hours  
6.6   : 2 hours

## Progression of project
- Improved unit testing, specially around iterative deepening
- Win check improvement. Now checking only if previously moved player has won.
- Move ordering. Now columns are tried in priority order, based on distance from center column.
- Heuristics is now symmetric.
- Heuristic supports checking if three in row empty can actually be player (immediate threat or future threat).
- More comprehensive code documentation

## Learnings
- I haven't ever heard of Null Window Search. I don't yet know exactly how that is going to work, if at all, 
but going to look it up.

## Planned next
- Null Window Search?
- Maybe improved integration tests
- Better heuristic evaluation. 

## Difficulties or unclear topics
- None

## Questions or feedback to lecturers and assistants
- None