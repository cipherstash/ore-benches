import {
  KMSClient,
  EncryptCommand,
  DecryptCommand,
} from "@aws-sdk/client-kms";
import type { EncryptionBackend, PlainRecord, EncRecord, OpStats } from "./types";
import { FIELDS } from "./types";

/**
 * AWS KMS backend — direct KMS Encrypt/Decrypt per value.
 *
 * AWS KMS has no bulk API, so a batch of N records × 3 fields is N×3 separate
 * KMS calls. We fire them all concurrently (Promise.all) — the most charitable
 * fan-out for AWS — but it's still one round-trip per value vs ZeroKMS's one
 * round-trip per batch. This is the per-value-mediation comparison (every
 * value individually auditable/revocable); see README "Fairness".
 */
class AwsKmsBackend implements EncryptionBackend {
  readonly name = "aws-kms" as const;
  private client!: KMSClient;
  private keyId!: string;

  async init(): Promise<void> {
    const keyId = process.env.AWS_KMS_KEY_ID;
    if (!keyId) throw new Error("AWS_KMS_KEY_ID is required for the aws-kms backend");
    this.keyId = keyId;
    this.client = new KMSClient({ region: process.env.AWS_REGION });
  }

  private async encryptOne(plaintext: string): Promise<string> {
    const res = await this.client.send(
      new EncryptCommand({ KeyId: this.keyId, Plaintext: Buffer.from(plaintext, "utf-8") }),
    );
    if (!res.CiphertextBlob) throw new Error("aws-kms encrypt returned no blob");
    return Buffer.from(res.CiphertextBlob).toString("base64");
  }

  private async decryptOne(ciphertext: string): Promise<string> {
    const res = await this.client.send(
      new DecryptCommand({ KeyId: this.keyId, CiphertextBlob: Buffer.from(ciphertext, "base64") }),
    );
    if (!res.Plaintext) throw new Error("aws-kms decrypt returned no plaintext");
    return Buffer.from(res.Plaintext).toString("utf-8");
  }

  async encryptBatch(input: PlainRecord[], stats?: OpStats): Promise<EncRecord[]> {
    const flat = await Promise.all(
      input.flatMap((r) => FIELDS.map((f) => this.encryptOne(r[f]))),
    );
    if (stats) stats.kmsCalls += flat.length; // one KMS Encrypt per value
    return regroup(flat);
  }

  async decryptBatch(input: EncRecord[], stats?: OpStats): Promise<PlainRecord[]> {
    const flat = await Promise.all(
      input.flatMap((r) => FIELDS.map((f) => this.decryptOne(r[f]))),
    );
    if (stats) stats.kmsCalls += flat.length; // one KMS Decrypt per value
    return regroup(flat);
  }
}

/** Reassemble a flat [v0_email, v0_name, v0_phone, v1_email, ...] array into records. */
export function regroup<T extends Record<string, string>>(flat: string[]): T[] {
  const out: T[] = [];
  for (let i = 0; i < flat.length; i += FIELDS.length) {
    out.push(
      Object.fromEntries(FIELDS.map((f, j) => [f, flat[i + j]])) as T,
    );
  }
  return out;
}

export function createAwsKmsBackend(): EncryptionBackend {
  return new AwsKmsBackend();
}
