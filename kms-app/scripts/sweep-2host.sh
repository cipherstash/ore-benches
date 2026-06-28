#!/usr/bin/env bash
# Two-host orchestrator — runs on the LOAD instance (B). For each cell it
# restarts the app server on instance A (over SSH, per-cell isolation) and runs
# Artillery against A's private IP, so the load generator never shares CPU with
# the system under test. A must already have .env.local + db:setup done.
#
# Env: A_IP (app private IP, required); KEY (ssh key, default ~/kms-bench.pem);
#      ROUNDS (3), DW (3), DS (15).
set -u
A_IP="${A_IP:?set A_IP to the app instance private IP}"
KEY="${KEY:-$HOME/kms-bench.pem}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"; cd "$HERE"
SSHO=(-i "$KEY" -o StrictHostKeyChecking=no -o ConnectTimeout=10 -o ServerAliveInterval=30)
TARGET="http://$A_IP:3000"
ROUNDS="${ROUNDS:-3}"; DW="${DW:-3}"; DS="${DS:-15}"
mkdir -p results/sweep results/throughput

restart_server(){ # backend — kill A's server, start fresh, wait healthy
  ssh "${SSHO[@]}" ec2-user@"$A_IP" "pkill -f next-server >/dev/null 2>&1; pkill -f 'next start' >/dev/null 2>&1; sleep 1" >/dev/null 2>&1 || true
  ssh "${SSHO[@]}" ec2-user@"$A_IP" "cd /opt/benches/kms-app && nohup env ENCRYPTION_BACKEND=$1 ENVELOPE_DATA_KEY_MAX_USES=1 npx next start -p 3000 -H 0.0.0.0 >/tmp/srv.log 2>&1 </dev/null &" >/dev/null 2>&1 || true
  for i in $(seq 1 90); do curl -sf "$TARGET/api/health" >/dev/null 2>&1 && return 0; sleep 1; done
  echo "  !! server failed to come up for $1"; return 1
}

write_cfg(){ # kind value rate file
  { echo "config:"; echo "  target: \"$TARGET\""; echo "  plugins: { metrics-by-endpoint: {} }";
    echo "  phases:"; echo "    - { name: warmup, duration: $DW, arrivalRate: $3 }";
    echo "    - { name: steady, duration: $DS, arrivalRate: $3 }";
    echo "  defaults: { headers: { content-type: application/json } }"; echo "scenarios:";
    if [ "$1" = insert ]; then echo "  - flow: [ { post: { name: insert, url: \"/api/records/insert\", json: { count: $2 } } } ]";
    else echo "  - flow: [ { get: { name: query, url: \"/api/records/query?limit=$2\" } } ]"; fi; } > "$4"
}
rate_for(){ case $1 in 20) echo 10;; 100) echo 5;; 500) echo 2;; *) echo 1;; esac; }

echo "### LATENCY ($ROUNDS rounds) ###"
for r in $(seq 1 "$ROUNDS"); do
  for b in zerokms aws-kms aws-kms-envelope; do
    for kind in insert query; do
      for s in 20 100 500 1000; do
        restart_server "$b" || continue
        write_cfg "$kind" "$s" "$(rate_for "$s")" /tmp/cfg.yml
        npx artillery run /tmp/cfg.yml -o "results/sweep/$kind-s$s-$b-r$r.json" >/tmp/art.log 2>&1
        node -e "const a=require('$HERE/results/sweep/$kind-s$s-$b-r$r.json').aggregate,c=a.counters||{},rt=a.summaries?.['http.response_time']||{};console.log('  r$r $b $kind s$s -> p95='+(rt.p95??'?')+' failed='+(c['vusers.failed']||0))" 2>/dev/null
      done
    done
  done
done

echo "### THROUGHPUT (batch 100, rates 50-800) ###"
BATCH=100; RATES="50 100 200 400 800"
echo "kind,batch,rate,backend,ok,failed,achieved_vps,offered_vps" > results/throughput/data.csv
for b in zerokms aws-kms aws-kms-envelope; do
  for kind in insert query; do
    for rate in $RATES; do
      restart_server "$b" || continue
      write_cfg "$kind" "$BATCH" "$rate" /tmp/cfg.yml
      npx artillery run /tmp/cfg.yml -o "results/throughput/$kind-b$BATCH-rate$rate-$b.json" >/tmp/art.log 2>&1
      row=$(node -e "const a=require('$HERE/results/throughput/$kind-b$BATCH-rate$rate-$b.json').aggregate,c=a.counters||{};const ok=(c['http.codes.200']||0)+(c['http.codes.201']||0),failed=c['vusers.failed']||0;const ach=Math.round(ok*$BATCH*3/($DW+$DS)),off=$rate*$BATCH*3;console.log(['$kind',$BATCH,$rate,'$b',ok,failed,ach,off].join(','))" 2>/dev/null)
      echo "$row" >> results/throughput/data.csv
      echo "  thru $b $kind rate=$rate -> $(echo "$row" | cut -d, -f7) v/s (failed $(echo "$row" | cut -d, -f6))"
    done
  done
done

ssh "${SSHO[@]}" ec2-user@"$A_IP" "pkill -f 'next start' || true" >/dev/null 2>&1 || true
node scripts/collect.mjs; node scripts/chart.mjs; node scripts/aggregate.mjs "$ROUNDS"
node scripts/throughput-chart.mjs
echo "BENCH_DONE_2HOST"
