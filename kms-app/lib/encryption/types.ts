/**
 * A pluggable encryption backend, batch-oriented.
 *
 * The benchmark's whole point is bulk amortization: a realistic request
 * encrypts/decrypts MANY values at once (e.g. 20 records × 3 fields = 60).
 * ZeroKMS does that in one network round-trip via its bulk API; AWS KMS has no
 * bulk API, so under per-value mediation it makes one call per value.
 *
 * Backends are therefore batch-in / batch-out. The harness swaps them via the
 * ENCRYPTION_BACKEND env var so Artillery can load-test each under an
 * otherwise-identical app and database.
 */
export const FIELDS = ["email", "name", "phone"] as const;
export type Field = (typeof FIELDS)[number];

/** A plaintext record — three encrypted fields. */
export type PlainRecord = Record<Field, string>;

/** An encrypted record — each field is a serialized ciphertext string (for a TEXT column). */
export type EncRecord = Record<Field, string>;

/** Per-request key-service work, for the data-key-reuse instrumentation:
 *  `kmsCalls` = round-trips to the key service (KMS Encrypt/Decrypt/GenerateDataKey,
 *  or ZeroKMS bulk calls). The whole point of the reuse experiment is that for a
 *  scattered query this equals the number of *distinct data keys* in the result. */
export type OpStats = { kmsCalls: number };

export interface EncryptionBackend {
  /** Backend identifier, also written to the `backend` column for traceability. */
  readonly name: "zerokms" | "aws-kms" | "aws-kms-envelope";
  /** One-time async setup (client construction, schema registration). */
  init(): Promise<void>;
  /** Encrypt a batch of records. One bulk operation for ZeroKMS; N×fields calls for AWS. */
  encryptBatch(records: PlainRecord[], stats?: OpStats): Promise<EncRecord[]>;
  /** Decrypt a batch of records. One bulk operation for ZeroKMS; N×fields calls for AWS. */
  decryptBatch(records: EncRecord[], stats?: OpStats): Promise<PlainRecord[]>;
}
