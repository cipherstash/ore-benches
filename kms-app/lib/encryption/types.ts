/**
 * A pluggable encryption backend. Each implementation encrypts/decrypts a
 * single field value and returns a serialized ciphertext string suitable for
 * storage in a Postgres `text` column. The harness swaps backends via the
 * ENCRYPTION_BACKEND env var so Artillery can load-test each under an
 * otherwise-identical app and database.
 */
export type Field = "email" | "name";

export interface EncryptionBackend {
  /** Backend identifier, also written to the `backend` column for traceability. */
  readonly name: "zerokms" | "aws-kms";
  /** One-time async setup (client construction, schema registration). */
  init(): Promise<void>;
  /** Encrypt a plaintext field value; returns a string to store. */
  encrypt(plaintext: string, field: Field): Promise<string>;
  /** Decrypt a stored ciphertext string back to plaintext. */
  decrypt(ciphertext: string, field: Field): Promise<string>;
}
