# Weekly report

## Summary
This week the core algorithm with alpha beta pruning is completed. Test coverage research done.

### Description
I have mostly put things together with the algorithm and connect four game engine. I spent some time to figure
out how test coverage can be accomplished in rust. I can now run test coverage reports from command line, but
nothing has been yet integrated nor automated. I also did a bit of code commenting and tried to generate documentation 
from the code. Seems to be working, but nothing yet integrated or automated.

I was thinking quite a bit how I should be approaching incremental improvements to the algorithm as one of the
requirements from course is to have several optimisations on top of the base minimax with alpha beta. I decided to 
build a small benchmark application that can execute repeated runs of the algorithm to reduce random variation 
when comparing optimisation results.

Building this benchmark application was relatively quick but required some small changes to the ai and engine crates.
I haven't yet found a good place to put the results as part of the documentation, but will do.

I feel I have now enough tools and code, so I can focus on the algorithm improvements on top of the baseline. I plan
to build the heuristics part next, have it unit tested and run a heuristics version of the algorithm against
the baseline and compare the results. 

## Time spent
17.5  : 3 hours  
20.5  : 2 hours  
22.5  : 2 hours  
23.5  : 3 hours

## Progression of project
- Implementation of pure minimax algorithm completed
- Implementation of alpha - beta pruning completed
- Mock state finished and unit tests completed
- Research and set up for code coverage completed (not yet fully integrated, nor automated) 
- Code documentation can be generated

## Learnings
- How to do code coverage reports in rust
- What crates these days have simple and effective command line parsing support

## Planned next
- Minimax heuristics implementation. Current minimax doesn't have any heuristics.
- Parametrisation: Time based constraining. This one of the means to measure improvements in the algorithm.
- Iterative deepening. This works well with time based constraining.
- Move ordering. Try middle ones first, move towards the perimeter of the board.
- Win check improvements: Check only rows that can contain the previous move, it must be part of the win row.

## Difficulties or unclear topics
- Borrow checker occasionally bites. I am not aiming for pure functional approach, so mutability causes small issues 
sometimes


## Questions or feedback to lecturers and assistants
- None