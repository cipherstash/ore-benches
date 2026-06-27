import { Encryption } from "@cipherstash/stack";
import { encryptedTable, encryptedColumn } from "@cipherstash/stack/schema";
import type { EncryptionBackend, PlainRecord, EncRecord } from "./types";
import { FIELDS } from "./types";

/**
 * ZeroKMS backend, via the CipherStash Encryption SDK (`@cipherstash/stack`).
 *
 * Uses bulkEncryptModels / bulkDecryptModels: per the SDK, each performs a
 * SINGLE call to ZeroKMS regardless of the number of models — so a batch of
 * 20 records × 3 fields is one network round-trip, not 60. This is where
 * ZeroKMS's throughput advantage over AWS KMS comes from.
 *
 * Credentials come from the local CipherStash profile (`stash login`,
 * ~/.cipherstash/) automatically — no env vars needed for local runs. For
 * headless/CI the FFI also reads CS_WORKSPACE_CRN / CS_CLIENT_ID /
 * CS_CLIENT_KEY / CS_CLIENT_ACCESS_KEY.
 */
const records = encryptedTable("records", {
  email: encryptedColumn("email"),
  name: encryptedColumn("name"),
  phone: encryptedColumn("phone"),
});

class ZeroKmsBackend implements EncryptionBackend {
  readonly name = "zerokms" as const;
  private client!: Awaited<ReturnType<typeof Encryption>>;

  async init(): Promise<void> {
    this.client = await Encryption({ schemas: [records] });
  }

  async encryptBatch(input: PlainRecord[]): Promise<EncRecord[]> {
    const result = await this.client.bulkEncryptModels(input, records);
    if (result.failure) {
      throw new Error(`zerokms bulkEncrypt failed: ${result.failure.message}`);
    }
    // Each model field is an EQL ciphertext object; serialize per field for TEXT storage.
    return result.data.map((m) => {
      const rec = m as Record<Field, unknown>;
      return Object.fromEntries(FIELDS.map((f) => [f, JSON.stringify(rec[f])])) as EncRecord;
    });
  }

  async decryptBatch(input: EncRecord[]): Promise<PlainRecord[]> {
    // Rebuild encrypted models from the stored ciphertext strings.
    const models = input.map((r) =>
      Object.fromEntries(FIELDS.map((f) => [f, JSON.parse(r[f])])),
    );
    const result = await this.client.bulkDecryptModels(models);
    if (result.failure) {
      throw new Error(`zerokms bulkDecrypt failed: ${result.failure.message}`);
    }
    return result.data.map((m) => {
      const rec = m as Record<Field, unknown>;
      return Object.fromEntries(FIELDS.map((f) => [f, String(rec[f])])) as PlainRecord;
    });
  }
}

// FIELDS is imported above; re-declare the element type locally for the maps.
type Field = (typeof FIELDS)[number];

export function createZeroKmsBackend(): EncryptionBackend {
  return new ZeroKmsBackend();
}
