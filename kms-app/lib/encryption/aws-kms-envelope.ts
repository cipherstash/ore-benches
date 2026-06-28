import {
  KMSClient,
  GenerateDataKeyCommand,
  DecryptCommand,
} from "@aws-sdk/client-kms";
import { createCipheriv, createDecipheriv, randomBytes } from "node:crypto";
import type { EncryptionBackend, PlainRecord, EncRecord, OpStats } from "./types";
import { FIELDS } from "./types";

/**
 * AWS KMS envelope encryption. KMS protects a local AES-256 data key (DEK); the
 * value is encrypted locally with AES-256-GCM.
 *
 * Data-key REUSE (ENVELOPE_DATA_KEY_MAX_USES > 1): one DEK encrypts many values
 * before a new one is generated, cutting GenerateDataKey calls on the write
 * path. This experiment shows reuse helps INGEST and SEQUENTIAL reads but not
 * SCATTERED reads — because a query's result set is keyed by *insert* locality,
 * which has nothing to do with *retrieval* order.
 *
 *   - WRITE: reuse requires holding the plaintext DEK and encrypting
 *     SEQUENTIALLY (concurrent fan-out would race getWriteKey and silently give
 *     each value its own DEK). The DEK persists across requests until exhausted.
 *   - READ: we de-duplicate the *distinct DEKs* in the result and KMS-Decrypt
 *     each once, PER REQUEST (a cold cache each query — no cross-query warm
 *     cache, which is the separable "caching" concern with cold-start + a
 *     growing pool of plaintext DEKs in memory). So kmsCalls == distinct DEKs.
 *
 * Reuse also weakens the model: one plaintext DEK in app memory now covers many
 * records, losing per-value audit/revocation.
 *
 * Stored ciphertext per field: JSON { edk, iv, tag, ct } (all base64).
 */
const ALGO = "aes-256-gcm";

type Field = (typeof FIELDS)[number];

class AwsKmsEnvelopeBackend implements EncryptionBackend {
  readonly name = "aws-kms-envelope" as const;
  private client!: KMSClient;
  private keyId!: string;
  private maxUses!: number;
  private writeKey: { plaintext: Buffer; encrypted: Buffer; uses: number } | null = null;

  async init(): Promise<void> {
    const keyId = process.env.AWS_KMS_KEY_ID;
    if (!keyId) throw new Error("AWS_KMS_KEY_ID is required for the aws-kms-envelope backend");
    this.keyId = keyId;
    this.client = new KMSClient({ region: process.env.AWS_REGION });
    // 1 = per-value data key (fair equal-security default). >1 = reuse (faster
    // writes, weaker model). Counted in values; 300 ≈ one DEK per 100 records.
    this.maxUses = Math.max(1, Number(process.env.ENVELOPE_DATA_KEY_MAX_USES ?? 1));
  }

  private async getWriteKey(stats?: OpStats) {
    if (this.writeKey && this.writeKey.uses < this.maxUses) {
      this.writeKey.uses += 1;
      return this.writeKey;
    }
    const res = await this.client.send(
      new GenerateDataKeyCommand({ KeyId: this.keyId, KeySpec: "AES_256" }),
    );
    if (!res.Plaintext || !res.CiphertextBlob) throw new Error("GenerateDataKey returned no key");
    if (stats) stats.kmsCalls += 1;
    this.writeKey = { plaintext: Buffer.from(res.Plaintext), encrypted: Buffer.from(res.CiphertextBlob), uses: 1 };
    return this.writeKey;
  }

  private encryptOneWith(dek: { plaintext: Buffer; encrypted: Buffer }, plaintext: string): string {
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

  async encryptBatch(input: PlainRecord[], stats?: OpStats): Promise<EncRecord[]> {
    if (this.maxUses > 1) {
      // SEQUENTIAL so the cached DEK is actually reused across values.
      const out: EncRecord[] = [];
      for (const r of input) {
        const rec = {} as EncRecord;
        for (const f of FIELDS) rec[f] = this.encryptOneWith(await this.getWriteKey(stats), r[f]);
        out.push(rec);
      }
      return out;
    }
    // per-value (maxUses=1): fresh DEK per value, fanned out concurrently.
    return Promise.all(input.map(async (r) => {
      const rec = {} as EncRecord;
      for (const f of FIELDS) rec[f] = this.encryptOneWith(await this.getWriteKey(stats), r[f]);
      return rec;
    }));
  }

  async decryptBatch(input: EncRecord[], stats?: OpStats): Promise<PlainRecord[]> {
    // Parse all fields; collect the DISTINCT data keys in this result set.
    const parsed = input.map((r) =>
      Object.fromEntries(FIELDS.map((f) => [f, JSON.parse(r[f])])) as Record<Field, { edk: string; iv: string; tag: string; ct: string }>,
    );
    const distinct = [...new Set(parsed.flatMap((p) => FIELDS.map((f) => p[f].edk)))];
    // KMS-Decrypt each distinct DEK once (concurrently). This is the cost the
    // reuse experiment measures: scattered query => distinct ≈ N records.
    const dekFor = new Map<string, Buffer>();
    await Promise.all(distinct.map(async (edk) => {
      const res = await this.client.send(
        new DecryptCommand({ KeyId: this.keyId, CiphertextBlob: Buffer.from(edk, "base64") }),
      );
      if (!res.Plaintext) throw new Error("KMS Decrypt returned no data key");
      dekFor.set(edk, Buffer.from(res.Plaintext));
    }));
    if (stats) stats.kmsCalls += distinct.length;
    // AES-decrypt every value locally with its (now in-memory) DEK.
    return parsed.map((p) => {
      const rec = {} as PlainRecord;
      for (const f of FIELDS) {
        const { edk, iv, tag, ct } = p[f];
        const d = createDecipheriv(ALGO, dekFor.get(edk)!, Buffer.from(iv, "base64"));
        d.setAuthTag(Buffer.from(tag, "base64"));
        rec[f] = Buffer.concat([d.update(Buffer.from(ct, "base64")), d.final()]).toString("utf-8");
      }
      return rec;
    });
  }
}

export function createAwsKmsEnvelopeBackend(): EncryptionBackend {
  return new AwsKmsEnvelopeBackend();
}
