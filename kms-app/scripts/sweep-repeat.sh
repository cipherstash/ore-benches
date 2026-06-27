#!/usr/bin/env bash
# Interleaved repeat sweep: run ROUNDS rounds, rotating backends each round so
# temporal AWS-side variance (throttle state, network) is shared across them.
# Writes round-tagged JSON to results/sweep/. Aggregate with scripts/aggregate.mjs.
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
ROUNDS="${ROUNDS:-3}"
BACKENDS=(zerokms aws-kms aws-kms-envelope)
export DS="${DS:-12}" DW="${DW:-2}" AWS_MAX_ATTEMPTS="${AWS_MAX_ATTEMPTS:-3}"

for r in $(seq 1 "$ROUNDS"); do
  echo "##################### ROUND $r/$ROUNDS #####################"
  for b in "${BACKENDS[@]}"; do
    ROUND="$r" bash scripts/sweep.sh "$b"
  done
done
echo "##################### SWEEP COMPLETE #####################"
