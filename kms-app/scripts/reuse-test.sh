#!/usr/bin/env bash
# Data-key REUSE experiment — runs on load instance B, drives app instance A.
# Demonstrates: reuse speeds INGEST and SEQUENTIAL reads, but a SCATTERED read
# (realistic retrieval pattern) touches ~one data key per record, so reuse's
# amortisation collapses. ZeroKMS is flat (one bulk call, pattern-independent).
#
# Env: A_IP (required), KEY (~/kms-bench.pem), QLIMIT (50 records/query).
set -u
A_IP="${A_IP:?set A_IP}"; KEY="${KEY:-$HOME/kms-bench.pem}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
SSHO=(-i "$KEY" -o StrictHostKeyChecking=no -o BatchMode=yes -o ConnectTimeout=10)
TARGET="http://$A_IP:3000"; QLIMIT="${QLIMIT:-50}"
OUT=results/reuse; mkdir -p "$OUT"
echo "phase,backend,maxuses,pattern,p95_ms,failed,kms_calls" > "$OUT/data.csv"

start_server(){ # backend maxuses
  ssh "${SSHO[@]}" ec2-user@"$A_IP" "sudo systemctl stop kmsapp 2>/dev/null; sleep 1; sudo systemd-run --unit=kmsapp --collect --working-directory=/opt/benches/kms-app --setenv=ENCRYPTION_BACKEND=$1 --setenv=ENVELOPE_DATA_KEY_MAX_USES=$2 /usr/bin/npx next start -p 3000 -H 0.0.0.0" >/dev/null 2>&1
  for i in $(seq 1 90); do curl -sf "$TARGET/api/health" >/dev/null 2>&1 && return 0; sleep 1; done; return 1
}
truncate_db(){ ssh "${SSHO[@]}" ec2-user@"$A_IP" "psql postgres://postgres:postgres@localhost:5432/postgres -q -c 'TRUNCATE records RESTART IDENTITY;'" >/dev/null 2>&1; }
jget(){ node -e "let d='';process.stdin.on('data',c=>d+=c).on('end',()=>{try{console.log(JSON.parse(d).$1??'?')}catch{console.log('?')}})"; }
p95_of(){ node -e "console.log((require('$1').aggregate.summaries['http.response_time']||{}).p95??'?')" 2>/dev/null; }
fail_of(){ node -e "console.log(require('$1').aggregate.counters['vusers.failed']||0)" 2>/dev/null; }

run_art(){ # file out
  npx artillery run "$1" -o "$2" >/dev/null 2>&1; }

# backend  maxuses  seed_rate  seed_target  label
CONFIGS=(
  "zerokms 1 30 100000 zerokms"
  "aws-kms-envelope 300 30 100000 envelope-reuse"
  "aws-kms-envelope 1 3 3000 envelope-per-value"
)

echo "### PHASE 1 — INGEST (does reuse help writes?) ###"
for c in "${CONFIGS[@]}"; do
  set -- $c; b=$1; mu=$2; rate=$3; label=$5
  truncate_db; start_server "$b" "$mu" || { echo "  $label: server failed"; continue; }
  cat > /tmp/ing.yml <<YAML
config: { target: "$TARGET", plugins: { metrics-by-endpoint: {} }, phases: [ { duration: 18, arrivalRate: $rate } ], defaults: { headers: { content-type: application/json } } }
scenarios: [ { flow: [ { post: { url: "/api/records/insert", json: { count: 100 } } } ] } ]
YAML
  run_art /tmp/ing.yml /tmp/ing.json
  kc=$(curl -s -X POST "$TARGET/api/records/insert" -H 'content-type: application/json' -d '{"count":100}' | jget kmsCalls)
  echo "ingest,$b,$mu,-,$(p95_of /tmp/ing.json),$(fail_of /tmp/ing.json),$kc" >> "$OUT/data.csv"
  echo "  $label: p95=$(p95_of /tmp/ing.json)ms  failed=$(fail_of /tmp/ing.json)  kms/100-rec-insert=$kc"
done

echo "### PHASE 2 — QUERY sequential vs scattered (does reuse help reads?) ###"
for c in "${CONFIGS[@]}"; do
  set -- $c; b=$1; mu=$2; srate=$3; starget=$4; label=$5
  truncate_db; start_server "$b" "$mu" || continue
  echo "  seeding $label to ~$starget rows..."
  dur=$(( starget/100/srate + 4 ))
  cat > /tmp/seed.yml <<YAML
config: { target: "$TARGET", phases: [ { duration: $dur, arrivalRate: $srate } ], defaults: { headers: { content-type: application/json } } }
scenarios: [ { flow: [ { post: { url: "/api/records/insert", json: { count: 100 } } } ] } ]
YAML
  run_art /tmp/seed.yml /tmp/seed.json
  start_server "$b" "$mu" || continue   # fresh idRange after seeding
  for pat in sequential scattered; do
    sc=false; [ "$pat" = scattered ] && sc=true
    cat > /tmp/q.yml <<YAML
config: { target: "$TARGET", plugins: { metrics-by-endpoint: {} }, phases: [ { duration: 20, arrivalRate: 50 } ] }
scenarios: [ { flow: [ { get: { url: "/api/records/query?limit=$QLIMIT&scatter=$sc" } } ] } ]
YAML
    run_art /tmp/q.yml /tmp/q.json
    kc=$(curl -s "$TARGET/api/records/query?limit=$QLIMIT&scatter=$sc" | jget kmsCalls)
    echo "query,$b,$mu,$pat,$(p95_of /tmp/q.json),$(fail_of /tmp/q.json),$kc" >> "$OUT/data.csv"
    echo "  $label $pat: p95=$(p95_of /tmp/q.json)ms  failed=$(fail_of /tmp/q.json)  kms/$QLIMIT-rec-query=$kc"
  done
done

ssh "${SSHO[@]}" ec2-user@"$A_IP" "sudo systemctl stop kmsapp 2>/dev/null || true" >/dev/null 2>&1 || true
echo "### REUSE TEST DONE ###"; echo; column -t -s, "$OUT/data.csv"
