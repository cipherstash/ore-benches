-- Schema for the KMS comparison harness.
-- Each record has three encrypted fields. Both backends store a serialized
-- ciphertext string per field, so the table shape is identical regardless of
-- which backend is under test. `backend` records which one wrote the row, so
-- the query benchmark only reads rows it can decrypt.

CREATE TABLE IF NOT EXISTS records (
  id               BIGSERIAL PRIMARY KEY,
  backend          TEXT NOT NULL,
  email_encrypted  TEXT NOT NULL,
  name_encrypted   TEXT NOT NULL,
  phone_encrypted  TEXT NOT NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The query benchmark selects random id windows filtered by backend.
CREATE INDEX IF NOT EXISTS records_backend_id_idx ON records (backend, id);
