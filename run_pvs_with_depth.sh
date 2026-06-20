#!/bin/bash
for START in 0 1 2 3 4 5; do
  echo "Running with PVS_START_DEPTH=$START"
  cargo run -p benchmark --release -- \
    --games 50 \
    --depth 9 \
    --heuristic-max v2 \
    --heuristic-min v2 \
    --pvs-start-depth "$START" \
    --output "results_pvs_start_${START}.json"
done
