# Weekly report

## Summary
Setup of the project. Feedback discussion with the course lecturer. Writing Connect 4 engine and finishing the 
requirements specification documentation according to the course plan.

### Description
I started to work on the project before the first week. This report contains the time spent before the first week as 
well as it contributes to the overall progress of the project. I had thought about implementing a game AI algorithm that 
I haven't done before using Rust programming language. Original idea was to compare Minimax and Monte Carlo Tree Search 
algorithms in Connect 4 game context. I had a feedback discussion with the lecturer and based on the feedback I am focusing
now on just Minimax algorithm with some optimisation techniques in mind and analysing how those techniques impact the
algorithm. 

Before the feedback discussion I already spent good amount of time to work with the Connect 4 engine, unit tests and
a simple text based UI for the engine. This is currently in pretty good shape, but not yet completed.

As part of the weeks goals to write the implementation specification I was looking into source materials and read some
algorithm (Minimax and MCTS) related information and documentation from the internet. Based on the initial research I 
wrote the requirements specification and set up the project in the Github and the Labtool.

## Time spent
7.5  : 6 hours  
8.5  : 2 hours  
14.5 : x hours  


## Progression of project
- Week 1 goals achieved
- GitHub and Labtool are set up
- Reading about the algorithms (Minimax and MCTS)
- Connect 4 engine implementation started and almost finished

## Learnings
- Refreshed on how to do Minimax with alpha-beta pruning on the idea level (I have done minimax before)
- Refreshed on some Rust ecosystem concepts and tooling like workspace etc.
- Bitboard implementation with checking the win condition with bitwise shifting (bit-shifting)
- Importance of the heuristic function to the Minimax algorithm

## Planned next
- Tic Tac Toe game state, board etc. for purely testing the Minimax implementation
- Some abstractions needed for AI to be able to use both games states (Connect 4 and Tic Tac Toe)
- Minimax implementation without alpha-beta pruning
- Testing and validation that with full depth search two minimax players will always tie from all possible start 
positions (Tic Tac Toe)
- Alpha-beta pruning implementation (Tic Tac Toe)
- Testing and validation that minimax algorithm with and without alpha-beta pruning yield the exact same results
- Finishing the Connect 4 game engine
- Moving the Minimax with alpha-beta to Connect 4 game context

## Difficulties or unclear topics
- Time management

## Questions or feedback to lecturers and assistants
- None