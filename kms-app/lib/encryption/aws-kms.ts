import {
  KMSClient,
  EncryptCommand,
  DecryptCommand,
} from "@aws-sdk/client-kms";
import type { EncryptionBackend, Field } from "./types";

/**
 * AWS KMS backend — the naive "encrypt the field directly with KMS" approach.
 *
 * This calls KMS Encrypt/Decrypt per value, which is the simplest way to use
 * KMS for application data and the pattern most teams reach for first. Note
 * the fairness caveats (see kms-app/README.md):
 *   - KMS Encrypt has a 4 KB plaintext limit and is rate-limited per region.
 *   - The production-grade AWS pattern is *envelope encryption* (KMS protects
 *     a local data key; AES-GCM encrypts the data). That trades a network
 *     round-trip per value for local crypto. A future backend variant
 *     (`aws-kms-envelope`) should be added to compare that path too.
 */
class AwsKmsBackend implements EncryptionBackend {
  readonly name = "aws-kms" as const;
  private client!: KMSClient;
  private keyId!: string;

  async init(): Promise<void> {
    const keyId = process.env.AWS_KMS_KEY_ID;
    if (!keyId) {
      throw new Error("AWS_KMS_KEY_ID is required for the aws-kms backend");
    }
    this.keyId = keyId;
    this.client = new KMSClient({ region: process.env.AWS_REGION });
  }

  async encrypt(plaintext: string, _field: Field): Promise<string> {
    const res = await this.client.send(
      new EncryptCommand({
        KeyId: this.keyId,
        Plaintext: Buffer.from(plaintext, "utf-8"),
      }),
    );
    if (!res.CiphertextBlob) throw new Error("aws-kms encrypt returned no blob");
    return Buffer.from(res.CiphertextBlob).toString("base64");
  }

  async decrypt(ciphertext: string, _field: Field): Promise<string> {
    const res = await this.client.send(
      new DecryptCommand({
        KeyId: this.keyId,
        CiphertextBlob: Buffer.from(ciphertext, "base64"),
      }),
    );
    if (!res.Plaintext) throw new Error("aws-kms decrypt returned no plaintext");
    return Buffer.from(res.Plaintext).toString("utf-8");
  }
}

export function createAwsKmsBackend(): EncryptionBackend {
  return new AwsKmsBackend();
}
