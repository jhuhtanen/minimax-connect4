#!/bin/bash
echo "Only works if pvs-start-depth is part of run config!"
for START in 0 1 2 3 4 5 6 7; do
  echo "Running with PVS_START_DEPTH=$START"
  cargo run -p benchmark --release -- \
    --games 50 \
    --depth 7 \
    --pvs-start-depth "$START" \
    --output "results_pvs_start_${START}.json"
done
