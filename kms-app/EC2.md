# Running the benchmark on EC2 (in-region, headline numbers)

The laptop run is a *dev-environment baseline* — home WiFi adds latency and
jitter to every round-trip, which **overstates** the gap (ZeroKMS pays that
overhead once per batch; AWS pays it per value). The publishable numbers should
come from an instance **in the same region as both services** (`ap-southeast-2`
— that's where this account's KMS key and ZeroKMS both live), so the only paths
measured are app→KMS and app→ZeroKMS, both in-region.

Run the app, Postgres, and the Artillery generator all on the one instance.

## 1. Instance role for KMS (no static secret)

Create a role the instance assumes, scoped to the 3 KMS actions on the one key:

```bash
KEY_ARN="arn:aws:kms:ap-southeast-2:688148311063:key/fba4b39f-2b91-4e6e-b63f-ef5f547fc083"

cat > /tmp/trust.json <<'JSON'
{ "Version": "2012-10-17", "Statement": [
  { "Effect": "Allow", "Principal": { "Service": "ec2.amazonaws.com" }, "Action": "sts:AssumeRole" } ] }
JSON
cat > /tmp/kms.json <<JSON
{ "Version": "2012-10-17", "Statement": [
  { "Effect": "Allow", "Action": ["kms:Encrypt","kms:Decrypt","kms:GenerateDataKey"], "Resource": "$KEY_ARN" } ] }
JSON

aws iam create-role --role-name kms-benchmark-ec2 --assume-role-policy-document file:///tmp/trust.json
aws iam put-role-policy --role-name kms-benchmark-ec2 --policy-name kms-key-crypto-only --policy-document file:///tmp/kms.json
aws iam create-instance-profile --instance-profile-name kms-benchmark-ec2
aws iam add-role-to-instance-profile --instance-profile-name kms-benchmark-ec2 --role-name kms-benchmark-ec2
```
(The harness's KMS client uses the default credential chain, so it picks up the
instance role automatically — no `AWS_ACCESS_KEY_ID` on the box.)

## 2. Headless ZeroKMS credentials

The interactive `~/.cipherstash` profile won't be on the instance, so mint a
non-interactive access key for the workspace (run on your laptop, where you're
logged in):

```bash
stash access-keys create kms-benchmark-ec2
# -> note the CS_WORKSPACE_CRN / CS_CLIENT_ID / CS_CLIENT_KEY / CS_CLIENT_ACCESS_KEY
```

## 3. Launch the instance

Compute-optimized (stable CPU — avoid burstable t-series throttling), Amazon
Linux 2023, in `ap-southeast-2`, with the instance profile attached:

```bash
aws ec2 run-instances --region ap-southeast-2 \
  --image-id resolve:ssm:/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64 \
  --instance-type c6i.xlarge \
  --iam-instance-profile Name=kms-benchmark-ec2 \
  --instance-initiated-shutdown-behavior terminate \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=kms-benchmark}]' \
  --metadata-options 'HttpTokens=required' \
  --user-data file://scripts/ec2-userdata.sh
  # add --key-name / --security-group-ids for SSM or SSH access as you prefer
```

Prefer **SSM Session Manager** over SSH (no inbound ports). Add the
`AmazonSSMManagedInstanceCore` managed policy to `kms-benchmark-ec2` if so.

## 4. On the instance

`scripts/ec2-userdata.sh` installs Node 20 + Postgres 16, clones the repo, and
builds the harness. Then:

```bash
cd /opt/benches/kms-app
cat > .env.local <<ENV
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
AWS_REGION=ap-southeast-2
AWS_KMS_KEY_ID=fba4b39f-2b91-4e6e-b63f-ef5f547fc083
ENVELOPE_DATA_KEY_MAX_USES=1
CS_WORKSPACE_CRN=...
CS_CLIENT_ID=...
CS_CLIENT_KEY=...
CS_CLIENT_ACCESS_KEY=...
ENV
npm run db:setup
ROUNDS=3 DS=15 DW=3 bash scripts/sweep-repeat.sh
node scripts/collect.mjs && node scripts/chart.mjs && node scripts/aggregate.mjs 3
```

Copy `results/sweep/` back (CSV + JSONs + SVG), then commit as the headline
in-region dataset alongside the laptop baseline.

## 5. Teardown

```bash
aws ec2 terminate-instances --region ap-southeast-2 --instance-ids <id>
aws iam remove-role-from-instance-profile --instance-profile-name kms-benchmark-ec2 --role-name kms-benchmark-ec2
aws iam delete-instance-profile --instance-profile-name kms-benchmark-ec2
aws iam delete-role-policy --role-name kms-benchmark-ec2 --policy-name kms-key-crypto-only
aws iam delete-role --role-name kms-benchmark-ec2
stash access-keys revoke kms-benchmark-ec2
```
