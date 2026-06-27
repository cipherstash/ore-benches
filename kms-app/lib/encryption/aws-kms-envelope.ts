import {
  KMSClient,
  GenerateDataKeyCommand,
  DecryptCommand,
} from "@aws-sdk/client-kms";
import { createCipheriv, createDecipheriv, randomBytes } from "node:crypto";
import type { EncryptionBackend, PlainRecord, EncRecord } from "./types";
import { FIELDS } from "./types";
import { regroup } from "./aws-kms";

/**
 * AWS KMS envelope encryption.
 *
 * KMS protects a local AES-256 data key (DEK); the value is encrypted locally
 * with AES-256-GCM. Like direct KMS, AWS has no bulk API — a batch of N×3
 * values is N×3 separate KMS operations (fired concurrently).
 *
 * IMPORTANT — security model vs caching:
 *   - DEFAULT (ENVELOPE_DATA_KEY_MAX_USES=1): a fresh data key per value, so
 *     every operation is its own KMS call and each value's access is
 *     independently auditable/revocable — the EQUAL-SECURITY comparison vs
 *     ZeroKMS.
 *   - Caching (MAX_USES > 1) reuses one DEK across many records with its
 *     plaintext in app memory: FASTER but a WEAKER model (lose per-value
 *     audit/revocation). A different security posture, not a fair comparison.
 *
 * Stored ciphertext is a JSON string: { edk, iv, tag, ct } (all base64).
 */
const ALGO = "aes-256-gcm";
const READ_CACHE_MAX = 4096;

class AwsKmsEnvelopeBackend implements EncryptionBackend {
  readonly name = "aws-kms-envelope" as const;
  private client!: KMSClient;
  private keyId!: string;
  private maxUses!: number;

  private writeKey: { plaintext: Buffer; encrypted: Buffer; uses: number } | null = null;
  private readCache = new Map<string, Buffer>();

  async init(): Promise<void> {
    const keyId = process.env.AWS_KMS_KEY_ID;
    if (!keyId) throw new Error("AWS_KMS_KEY_ID is required for the aws-kms-envelope backend");
    this.keyId = keyId;
    this.client = new KMSClient({ region: process.env.AWS_REGION });
    // Default 1 = per-value data key (fair, equal-security). >1 = caching (weaker).
    this.maxUses = Math.max(1, Number(process.env.ENVELOPE_DATA_KEY_MAX_USES ?? 1));
  }

  private async getWriteKey() {
    if (this.writeKey && this.writeKey.uses < this.maxUses) {
      this.writeKey.uses += 1;
      return this.writeKey;
    }
    const res = await this.client.send(
      new GenerateDataKeyCommand({ KeyId: this.keyId, KeySpec: "AES_256" }),
    );
    if (!res.Plaintext || !res.CiphertextBlob) throw new Error("GenerateDataKey returned no key");
    this.writeKey = {
      plaintext: Buffer.from(res.Plaintext),
      encrypted: Buffer.from(res.CiphertextBlob),
      uses: 1,
    };
    return this.writeKey;
  }

  private async encryptOne(plaintext: string): Promise<string> {
    const dek = await this.getWriteKey();
    const iv = randomBytes(12);
    const cipher = createCipheriv(ALGO, dek.plaintext, iv);
    const ct = Buffer.concat([cipher.update(plaintext, "utf-8"), cipher.final()]);
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
      new DecryptCommand({ KeyId: this.keyId, CiphertextBlob: Buffer.from(edkB64, "base64") }),
    );
    if (!res.Plaintext) throw new Error("KMS Decrypt returned no data key");
    const dek = Buffer.from(res.Plaintext);
    if (this.readCache.size >= READ_CACHE_MAX) {
      this.readCache.delete(this.readCache.keys().next().value as string);
    }
    this.readCache.set(edkB64, dek);
    return dek;
  }

  private async decryptOne(ciphertext: string): Promise<string> {
    const { edk, iv, tag, ct } = JSON.parse(ciphertext);
    const dek = await this.getReadKey(edk);
    const decipher = createDecipheriv(ALGO, dek, Buffer.from(iv, "base64"));
    decipher.setAuthTag(Buffer.from(tag, "base64"));
    return Buffer.concat([
      decipher.update(Buffer.from(ct, "base64")),
      decipher.final(),
    ]).toString("utf-8");
  }

  async encryptBatch(input: PlainRecord[]): Promise<EncRecord[]> {
    const flat = await Promise.all(
      input.flatMap((r) => FIELDS.map((f) => this.encryptOne(r[f]))),
    );
    return regroup(flat);
  }

  async decryptBatch(input: EncRecord[]): Promise<PlainRecord[]> {
    const flat = await Promise.all(
      input.flatMap((r) => FIELDS.map((f) => this.decryptOne(r[f]))),
    );
    return regroup(flat);
  }
}

export function createAwsKmsEnvelopeBackend(): EncryptionBackend {
  return new AwsKmsEnvelopeBackend();
}
