#!/bin/bash

cargo run -p benchmark --release -- --heuristic-min=v1 --heuristic-max=v2 --games 50 --depth 6
