import {
  KMSClient,
  GenerateDataKeyCommand,
  DecryptCommand,
} from "@aws-sdk/client-kms";
import { createCipheriv, createDecipheriv, randomBytes } from "node:crypto";
import type { EncryptionBackend, Field } from "./types";

/**
 * AWS KMS envelope encryption — the production-grade AWS pattern.
 *
 * KMS protects a local AES-256 *data key* (DEK); the field value is encrypted
 * locally with AES-256-GCM. This removes the 4 KB plaintext limit of direct
 * KMS Encrypt, and — crucially — lets a single DEK protect many values, so we
 * are not bound to one KMS call per value.
 *
 * Data-key caching (the realistic optimization, à la the AWS Encryption SDK):
 *   - Write: reuse a cached DEK for up to ENVELOPE_DATA_KEY_MAX_USES values
 *     before calling GenerateDataKey again. Set it to 1 to force a KMS call
 *     per value (worst case, closest to the naive `aws-kms` backend).
 *   - Read: cache the decrypted plaintext DEK keyed by its encrypted form, so
 *     reading many values that share a DEK costs one KMS Decrypt, not N.
 *
 * Stored ciphertext is a JSON string: { edk, iv, tag, ct } (all base64).
 */
const ALGO = "aes-256-gcm";
const READ_CACHE_MAX = 1024; // bound the plaintext-DEK cache for long runs

class AwsKmsEnvelopeBackend implements EncryptionBackend {
  readonly name = "aws-kms-envelope" as const;
  private client!: KMSClient;
  private keyId!: string;
  private maxUses!: number;

  // Write-side DEK, reused up to `maxUses` times.
  private writeKey: { plaintext: Buffer; encrypted: Buffer; uses: number } | null =
    null;
  // Read-side cache of plaintext DEKs, keyed by base64(encrypted DEK).
  private readCache = new Map<string, Buffer>();

  async init(): Promise<void> {
    const keyId = process.env.AWS_KMS_KEY_ID;
    if (!keyId) {
      throw new Error(
        "AWS_KMS_KEY_ID is required for the aws-kms-envelope backend",
      );
    }
    this.keyId = keyId;
    this.client = new KMSClient({ region: process.env.AWS_REGION });
    this.maxUses = Math.max(1, Number(process.env.ENVELOPE_DATA_KEY_MAX_USES ?? 1000));
  }

  private async getWriteKey() {
    if (this.writeKey && this.writeKey.uses < this.maxUses) {
      this.writeKey.uses += 1;
      return this.writeKey;
    }
    const res = await this.client.send(
      new GenerateDataKeyCommand({ KeyId: this.keyId, KeySpec: "AES_256" }),
    );
    if (!res.Plaintext || !res.CiphertextBlob) {
      throw new Error("GenerateDataKey returned no key material");
    }
    this.writeKey = {
      plaintext: Buffer.from(res.Plaintext),
      encrypted: Buffer.from(res.CiphertextBlob),
      uses: 1,
    };
    return this.writeKey;
  }

  async encrypt(plaintext: string, _field: Field): Promise<string> {
    const dek = await this.getWriteKey();
    const iv = randomBytes(12);
    const cipher = createCipheriv(ALGO, dek.plaintext, iv);
    const ct = Buffer.concat([
      cipher.update(plaintext, "utf-8"),
      cipher.final(),
    ]);
    return JSON.stringify({
      edk: dek.encrypted.toString("base64"),
      iv: iv.toString("base64"),
      tag: cipher.getAuthTag().toString("base64"),
      ct: ct.toString("base64"),
    });
  }

  private async getReadKey(edkB64: string): Promise<Buffer> {
    const cached = this.readCache.get(edkB64);
    if (cached) return cached;
    const res = await this.client.send(
      new DecryptCommand({
        KeyId: this.keyId,
        CiphertextBlob: Buffer.from(edkB64, "base64"),
      }),
    );
    if (!res.Plaintext) throw new Error("KMS Decrypt returned no data key");
    const dek = Buffer.from(res.Plaintext);
    if (this.readCache.size >= READ_CACHE_MAX) {
      // FIFO eviction — Map preserves insertion order.
      this.readCache.delete(this.readCache.keys().next().value as string);
    }
    this.readCache.set(edkB64, dek);
    return dek;
  }

  async decrypt(ciphertext: string, _field: Field): Promise<string> {
    const { edk, iv, tag, ct } = JSON.parse(ciphertext);
    const dek = await this.getReadKey(edk);
    const decipher = createDecipheriv(ALGO, dek, Buffer.from(iv, "base64"));
    decipher.setAuthTag(Buffer.from(tag, "base64"));
    return Buffer.concat([
      decipher.update(Buffer.from(ct, "base64")),
      decipher.final(),
    ]).toString("utf-8");
  }
}

export function createAwsKmsEnvelopeBackend(): EncryptionBackend {
  return new AwsKmsEnvelopeBackend();
}
