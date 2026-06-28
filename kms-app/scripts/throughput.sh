#!/usr/bin/env bash
# Throughput sweep for one backend. Usage: scripts/throughput.sh <backend>
# Holds a fixed batch size and steps the request rate up toward saturation,
# measuring achieved values/sec (encrypt or decrypt). Each cell runs against a
# fresh server (isolation). Insert cells seed the table for the query cells.
# Appends a tidy row per cell to results/throughput/data.csv.
#
# Throughput here is a *floor* on a laptop — the load generator/network may
# saturate before the backend does. Run on EC2 (in-region) for the real ceiling.
set -u
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
BACKEND="${1:?usage: throughput.sh <backend>}"
PORT=3321
BATCH="${BATCH:-100}"          # records per request (×3 fields = values/request)
RATES=(10 25 50 100)           # requests/sec offered
DW="${DW:-2}"; DS="${DS:-12}"
OUT=results/throughput; mkdir -p "$OUT"
CSV="$OUT/data.csv"
[ -f "$CSV" ] || echo "kind,batch,rate,backend,ok,failed,achieved_vps,offered_vps" > "$CSV"

write_cfg() { # kind rate file
  local kind=$1 rate=$2 file=$3
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
      echo "  - flow: [ { post: { name: insert, url: \"/api/records/insert\", json: { count: $BATCH } } } ]"
    else
      echo "  - flow: [ { get: { name: query, url: \"/api/records/query?limit=$BATCH\" } } ]"
    fi
  } > "$file"
}

run_cell() { # kind rate
  local kind=$1 rate=$2
  ENCRYPTION_BACKEND="$BACKEND" ENVELOPE_DATA_KEY_MAX_USES=1 \
    npx next start -p $PORT >/tmp/thru-$BACKEND.log 2>&1 &
  local svr=$!
  curl -s --retry-connrefused --retry 40 --retry-delay 1 -o /dev/null "http://localhost:$PORT/" 2>/dev/null
  write_cfg "$kind" "$rate" /tmp/thru-cfg.yml
  npx artillery run /tmp/thru-cfg.yml -o "$OUT/$kind-b$BATCH-rate$rate-$BACKEND.json" >/tmp/thru-art.log 2>&1
  kill "$svr" 2>/dev/null; wait "$svr" 2>/dev/null
  local row
  row=$(node -e "const a=require('$HERE/$OUT/$kind-b$BATCH-rate$rate-$BACKEND.json').aggregate,c=a.counters||{};
    const ok=(c['http.codes.200']||0)+(c['http.codes.201']||0),failed=c['vusers.failed']||0;
    const achieved=Math.round(ok*$BATCH*3/($DW+$DS)),offered=$rate*$BATCH*3;
    console.log(['$kind',$BATCH,$rate,'$BACKEND',ok,failed,achieved,offered].join(','))" 2>/dev/null)
  echo "$row" >> "$CSV"
  echo "  $kind rate=$rate -> achieved=$(echo "$row" | cut -d, -f7) values/s  (ok=$(echo "$row" | cut -d, -f5) failed=$(echo "$row" | cut -d, -f6))"
  sleep 2
}

echo "### throughput backend=$BACKEND batch=$BATCH ###"
for kind in insert query; do
  for r in "${RATES[@]}"; do run_cell "$kind" "$r"; done
done
echo "### done $BACKEND ###"
