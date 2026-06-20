# Weekly report

## Summary
I started to put all the variants into single function. I implemented the explicit
start depth parameter for PVS. 

### Description
I had a discussion with the lecturer about the results I got with PVS. Essentially the key discovery was that the PVS 
variant (or optimization, if you will) might actually visit more nodes depending on the state and
used start depth. For very shallow depth it can actually do double work as there's more nodes to be visited, and for very 
deep depth it might not be as useful since in high depth the subtrees are smaller.

Based on this I added a parameter to the PVS which defines at what depth we start to
actually use the PVS. We discussed that this could be hardcoded value but since I wanted to see what start depth makes sense
I parametrized it, and later it can be removed and hardcoded.

I compared the plain iterative deepening search with fixed depth of 9 without time limit. And PVS variants
with different pvs start depths. The results verify that PVS is pruning more effectively.

Iterative deepening (without PVS) results:

```
{
  "config": {
    "games": 50,
    "depth": 9,
    "time_ms": null,
    "alpha_beta": true,
    "heuristic_min": "V2",
    "heuristic_max": "V2",
    "pvs_start_depth": 0
  },
  "total_games": 50,
  "max_wins": 25,
  "min_wins": 25,
  "draws": 0,
  "avg_nodes_per_move": 169575.3,
  "avg_millis_per_move": 48.3735,
  "total_millis": 96747.0,
```

Iterative deepening with PVS, start depth of 4 results:

```
{
  "config": {
    "games": 50,
    "depth": 9,
    "time_ms": null,
    "alpha_beta": true,
    "heuristic_min": "V2",
    "heuristic_max": "V2",
    "pvs_start_depth": 4
  },
  "total_games": 50,
  "max_wins": 25,
  "min_wins": 25,
  "draws": 0,
  "avg_nodes_per_move": 85718.65,
  "avg_millis_per_move": 32.5625,
  "total_millis": 65125.0,
```
Some highlights: avg nodes per move are approximately half with PVS. Also time spent per move and total execution time are 
significantly better. (33%)

I have stored the results of the different start depth tests under docs.
The results of the runs summarized:

| PVS_START_DEPTH | max_wins | min_wins | draws | avg_nodes_per_move | Δ nodes vs 0 | avg_ms_per_move | Δ ms vs 0  |
|-----------------|----------|----------|-------|--------------------|-------------|-----------------|-----------|
| 0               | 25       | 25       | 0     | 87,586.675         | 0.00%       | 33.93           | 0.00%     |
| 1               | 25       | 25       | 0     | 87,586.675         | 0.00%       | 33.88           | -0.15%    |
| 2               | 25       | 25       | 0     | 86,457.425         | -1.29%      | 33.31           | -1.83%    |
| 3               | 25       | 25       | 0     | 85,974.650         | -1.84%      | 32.94           | -2.91%    |
| 4               | 25       | 25       | 0     | 85,718.650         | -2.13%      | 32.56           | -4.04%    |
| 5               | 25       | 25       | 0     | 87,231.975         | -0.41%      | 32.43           | -4.43%    |

## Time spent
14.6  : 3 hours  
17.6  : 1.5 hours  
20.6  : 5 hours  

## Progression of project
- Explicit start depth for PVS implemented
- Better tests for PVS vs plain iterative variants
- Started to refactor the variants into single function

## Learnings
- How explicit pvs start depth influences the results. For a UI based use, where time
limitation is needed, the fixed start depth do not really make sense. It looks like
the best start depth (for PVS) is relative to max depth.

## Planned next
- Based on discussions with the lecturer, I want to make sure the Minimax is tested properly
on the 5 moves to win and 3 moves to win scenarios.

## Difficulties or unclear topics
- None

## Questions or feedback to lecturers and assistants
- None