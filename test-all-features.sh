#!/usr/bin/env bash
set -euo pipefail

echo "Testing without any features"
cargo test --no-default-features

# read all arguments into an array
features=("send" "multithreaded" "scaffold")
n=${#features[@]}

# loop over all bitmasks from 1 to 2^n - 1 (skip 0 to omit the empty set)
for (( mask=1; mask < (1<<n); mask++ )); do
  subset=()

  # for each bit position, if it's set include that item
  for (( i=0; i<n; i++ )); do
    if (( mask & (1<<i) )); then
      subset+=("${features[i]}")
    fi
  done

  # Join with commas
  echo "Testing features ${subset[*]}"
  IFS=,; cargo test --no-default-features -F "${subset[*]}"
done

echo "All tests passed!"
