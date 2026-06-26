import { Encryption } from "@cipherstash/stack";
import { encryptedTable, encryptedColumn } from "@cipherstash/stack/schema";
import type { EncryptionBackend, Field } from "./types";

/**
 * ZeroKMS backend, via the CipherStash Encryption SDK (`@cipherstash/stack`).
 *
 * Each value is encrypted with a unique, per-record key managed by ZeroKMS.
 * The SDK returns a JSON payload (the EQL ciphertext envelope); we serialize
 * it to a string for storage and parse it back on read. This mirrors the
 * production SDK pattern documented at
 * https://cipherstash.com/docs/stack/reference/comparisons/aws-kms
 */
const users = encryptedTable("users", {
  email: encryptedColumn("email"),
  name: encryptedColumn("name"),
});

class ZeroKmsBackend implements EncryptionBackend {
  readonly name = "zerokms" as const;
  private client!: Awaited<ReturnType<typeof Encryption>>;

  async init(): Promise<void> {
    // Credentials are read from the environment by the SDK:
    // CS_CLIENT_ID, CS_CLIENT_KEY, CS_CLIENT_ACCESS_KEY, CS_WORKSPACE_CRN.
    this.client = await Encryption({ schemas: [users] });
  }

  async encrypt(plaintext: string, field: Field): Promise<string> {
    const result = await this.client.encrypt(plaintext, {
      column: users[field],
      table: users,
    });
    if (result.failure) {
      throw new Error(`zerokms encrypt failed: ${result.failure.message}`);
    }
    // result.data is the EQL JSON envelope; store it as a string.
    return JSON.stringify(result.data);
  }

  async decrypt(ciphertext: string, _field: Field): Promise<string> {
    const result = await this.client.decrypt(JSON.parse(ciphertext));
    if (result.failure) {
      throw new Error(`zerokms decrypt failed: ${result.failure.message}`);
    }
    // decrypt() returns JsPlaintext (string | number | …); we only ever encrypt
    // string field values, so coerce back to string.
    return String(result.data);
  }
}

export function createZeroKmsBackend(): EncryptionBackend {
  return new ZeroKmsBackend();
}
