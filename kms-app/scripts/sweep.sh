#!/usr/bin/env bash
# Batch-size sweep for one backend. Usage: scripts/sweep.sh <backend>
# Runs insert (write) then query (read) at sizes 20/100/500/1000, writing
# Artillery JSON to results/sweep/. Insert runs seed the table (persisted in
# Postgres) for the query runs.
#
# Each cell runs against a FRESH server process (started then killed), so an
# AWS large-batch meltdown — thousands of in-flight/retrying KMS calls — dies
# with its process and cannot spill timeouts into the next cell.
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
BACKEND="${1:?usage: sweep.sh <backend>}"
PORT=3320
SIZES=(20 100 500 1000)
DW="${DW:-3}"; DS="${DS:-22}"   # env-overridable warmup / steady durations
OUT=results/sweep; mkdir -p "$OUT"

rate_for() { case "$1" in 20) echo 10;; 100) echo 5;; 500) echo 2;; *) echo 1;; esac; }

write_cfg() { # kind size file
  local kind=$1 size=$2 file=$3 rate; rate=$(rate_for "$size")
  {
    echo "config:"
    echo "  target: \"http://localhost:$PORT\""
    echo "  plugins: { metrics-by-endpoint: {} }"
    echo "  phases:"
    echo "    - { name: warmup, duration: $DW, arrivalRate: $rate }"
    echo "    - { name: steady, duration: $DS, arrivalRate: $rate }"
    echo "  defaults: { headers: { content-type: application/json } }"
    echo "scenarios:"
    if [ "$kind" = insert ]; then
      echo "  - flow: [ { post: { name: insert, url: \"/api/records/insert\", json: { count: $size } } } ]"
    else
      echo "  - flow: [ { get: { name: query, url: \"/api/records/query?limit=$size\" } } ]"
    fi
  } > "$file"
}

run_cell() { # kind size
  local kind=$1 s=$2
  ENCRYPTION_BACKEND="$BACKEND" ENVELOPE_DATA_KEY_MAX_USES=1 \
    npx next start -p $PORT >/tmp/sweep-$BACKEND.log 2>&1 &
  local svr=$!
  curl -s --retry-connrefused --retry 40 --retry-delay 1 -o /dev/null "http://localhost:$PORT/" 2>/dev/null
  write_cfg "$kind" "$s" /tmp/sweep-cfg.yml
  local out="$OUT/$kind-s$s-$BACKEND${ROUND:+-r$ROUND}.json"
  npx artillery run /tmp/sweep-cfg.yml -o "$out" >/tmp/sweep-art.log 2>&1
  kill "$svr" 2>/dev/null; wait "$svr" 2>/dev/null
  node -e "const a=require('$HERE/$out').aggregate,c=a.counters,rt=a.summaries?.['http.response_time']||{};
    console.log('$kind size=$s'.padEnd(18)+'p95='+(rt.p95??'?')+'  failed='+(c['vusers.failed']||0)+'  ok='+((c['http.codes.200']||0)+(c['http.codes.201']||0)))" 2>/dev/null
  sleep 2
}

echo "### backend=$BACKEND ###"
for kind in insert query; do
  for s in "${SIZES[@]}"; do run_cell "$kind" "$s"; done
done
echo "### done $BACKEND ###"
