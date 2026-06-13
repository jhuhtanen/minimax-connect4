# Weekly report

## Summary
Most of my time this week was spent on peer review. I also implemented
Principal Variance Search.

### Description
I spent most of my Tuesday 9.6 reading, understanding and putting together a peer review
GitHub issue. It was refreshing to read some c++ code after a long time. I feel I managed to understand everything well
in the project and hopefully provided a lot of useful feedback.

At the same time I was waiting eagerly mine. I checked every day if there was a new issue
created to my repo but no issue has been crated at the time of writing this. (Saturday 21:38) 
This was clearly a disappointment for me. 

I also spent a lot of time to debug the Principal Variance Search. It seems
I made a too optimistic assumption when writing unit tests that it would be
an optimisation that in *all* cases it would yield less nodes visited than
it's plain alpha beta counterpart. My tests asserted that the score and the best
move would always be the same and this is true. But at the beginning my assumption
of also nodes visited being less turned out to be false. Great learning actually.

Additionally, I decided to side track a bit for creating a support for building
moves from a human-readable format of the board state. This is my attempt to be
able to verify if PVS version would work. I came into realization that this
might have been very useful much earlier.

## Time spent
7.6  : 4 hours  
9.6  : 6 hours  
11.6  : 3 hours  
13.6 : 5 hours

## Progression of project
- Principal Variance Search implemented (pvs variant as separate function)
- Invariance tests added

## Learnings
- Learned how to do Principal Variance Search
- Learned it most of the time results in fewer nodes visited, but not always.

## Planned next
- A conversation with the lecturer about testing
- Convert the iterative deepening to use PVS (now as separate function)
- Optional: Better heuristic evaluation.

## Difficulties or unclear topics
- None

## Questions or feedback to lecturers and assistants
- None