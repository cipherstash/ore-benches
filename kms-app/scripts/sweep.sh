#!/usr/bin/env bash
# Batch-size sweep for one backend. Usage: scripts/sweep.sh <backend>
# Runs insert (write) then query (read) at sizes 20/100/500/1000, writing
# Artillery JSON to results/sweep/. Insert runs also seed the table for query.
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
BACKEND="${1:?usage: sweep.sh <backend>}"
PORT=3320
DB="postgres://postgres:postgres@localhost:5400/postgres"
SIZES=(20 100 500 1000)
# Durations are env-overridable so AWS runs can be kept short enough to finish
# inside a short-lived SSO session. AWS_MAX_ATTEMPTS (SDK retry cap) is also
# passed through so throttled calls fail fast instead of retrying for ~30s.
DW="${DW:-3}"; DS="${DS:-22}"
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

echo "### backend=$BACKEND ###"
ENCRYPTION_BACKEND="$BACKEND" ENVELOPE_DATA_KEY_MAX_USES=1 npx next start -p $PORT >/tmp/sweep-$BACKEND.log 2>&1 &
SVR=$!
curl -s --retry-connrefused --retry 30 --retry-delay 1 -o /dev/null "http://localhost:$PORT/" 2>/dev/null
echo "health: $(curl -s http://localhost:$PORT/api/health)"

for kind in insert query; do
  for s in "${SIZES[@]}"; do
    write_cfg "$kind" "$s" /tmp/sweep-cfg.yml
    npx artillery run /tmp/sweep-cfg.yml -o "$OUT/$kind-s$s-$BACKEND.json" >/tmp/sweep-art.log 2>&1
    codes=$(grep -oE "http.codes.[0-9]+: +\.+ +[0-9]+" /tmp/sweep-art.log | tr -s ' ' | tr '\n' ' ')
    p95=$(grep -A6 "response_time" /tmp/sweep-art.log | grep -m1 "p95" | grep -oE "[0-9.]+$")
    failed=$(grep -m1 "vusers.failed" /tmp/sweep-art.log | grep -oE "[0-9]+$")
    printf "%-6s size=%-5s p95=%-8s failed=%-5s codes: %s\n" "$kind" "$s" "${p95:-?}" "${failed:-?}" "$codes"
    sleep 2  # let in-flight KMS calls settle before the next cell
  done
done
kill $SVR 2>/dev/null
echo "### done $BACKEND ###"
