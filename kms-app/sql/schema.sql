-- Schema for the KMS comparison harness.
-- Both backends store a serialized ciphertext string per encrypted field, so
-- the table shape is identical regardless of which backend is under test.
-- `backend` records which one wrote the row (zerokms | aws-kms).

CREATE TABLE IF NOT EXISTS users (
  id               BIGSERIAL PRIMARY KEY,
  backend          TEXT NOT NULL,
  email_encrypted  TEXT NOT NULL,
  name_encrypted   TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
