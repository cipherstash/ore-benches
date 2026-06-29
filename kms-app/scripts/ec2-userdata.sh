#!/usr/bin/env bash
# EC2 user-data: provision an Amazon Linux 2023 box to run the KMS benchmark.
# Installs Node 20 + Postgres 16, clones the repo, builds the harness. Finish
# the run manually per EC2.md (fill .env.local, db:setup, sweep-repeat.sh).
set -euxo pipefail

# Node 20
curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
dnf install -y nodejs git

# Postgres 16 (local — keeps the DB path off the network)
dnf install -y postgresql16 postgresql16-server
/usr/bin/postgresql-setup --initdb
systemctl enable --now postgresql
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"
# trust local connections so DATABASE_URL works without a password prompt
PGHBA=$(sudo -u postgres psql -tAc "SHOW hba_file;")
sed -i 's/^\(host.*127.0.0.1\/32.*\)\(ident\|scram-sha-256\|md5\)/\1trust/' "$PGHBA"
systemctl restart postgresql

# Repo + harness
mkdir -p /opt && cd /opt
git clone https://github.com/cipherstash/benches.git
cd benches/kms-app
npm install
npm run build

chown -R ec2-user:ec2-user /opt/benches
echo "Provisioned. Next: cd /opt/benches/kms-app; write .env.local (see EC2.md); npm run db:setup; ROUNDS=3 DS=15 DW=3 bash scripts/sweep-repeat.sh"
