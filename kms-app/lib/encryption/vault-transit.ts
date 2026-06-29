import { createCipheriv, createDecipheriv, randomBytes } from "node:crypto";
import type { EncryptionBackend, PlainRecord, EncRecord, OpStats } from "./types";
import { FIELDS } from "./types";

/**
 * HashiCorp Vault Transit backend. Vault Transit is the closest analog to
 * ZeroKMS — encryption-as-a-service — and, unlike AWS KMS, it HAS a batch API,
 * so a fair comparison must use it (handicapping Vault by ignoring `batch_input`
 * would repeat the very unfairness we avoid for AWS). Two modes, set by
 * VAULT_TRANSIT_MODE, map to the comparison's "trilemma":
 *
 *   - "direct" (default): the app sends PLAINTEXT to Vault; Vault encrypts and
 *     returns ciphertext. One `batch_input` round-trip for the whole batch
 *     (kmsCalls += 1), like ZeroKMS's bulk call. With VAULT_TRANSIT_DERIVED=true
 *     each record carries its own `context`, deriving a per-record key (point
 *     VAULT_TRANSIT_KEY at a key created with `derived=true`). The catch is that
 *     plaintext transits the Vault server.
 *
 *   - "envelope": Vault's `datakey` endpoint mints a data key (plaintext +
 *     Vault-wrapped copy); the value is encrypted LOCALLY (AES-256-GCM), so
 *     plaintext never leaves the client. `datakey` has NO batch API, so a unique
 *     key per record is one round-trip PER RECORD (slow writes); reuse
 *     (VAULT_DATA_KEY_MAX_USES > 1) amortizes writes but shares a key across many
 *     records (losing per-record audit/revocation). On READ, Vault unwraps all
 *     distinct data keys in ONE batched `decrypt` round-trip — so, unlike AWS
 *     KMS envelope, a scattered read does NOT collapse to one call per record.
 *     That divergence is a finding the benchmark is meant to surface.
 *
 * Stored ciphertext per field:
 *   - direct:   JSON { ct, ctx? }            (ctx = per-record derived context)
 *   - envelope: JSON { edk, iv, tag, ct }    (edk = Vault-wrapped data key string)
 */
const ALGO = "aes-256-gcm";
type Field = (typeof FIELDS)[number];

type DirectCell = { ct: string; ctx?: string };
type EnvelopeCell = { edk: string; iv: string; tag: string; ct: string };

class VaultTransitBackend implements EncryptionBackend {
  readonly name = "vault-transit" as const;
  private addr!: string;
  private token!: string;
  private key!: string;
  private mode!: "direct" | "envelope";
  private derived!: boolean; // direct mode: per-record context => per-record key
  private maxUses!: number; // envelope mode: data-key reuse (in values); 1 = per-value
  private writeKey: { plaintext: Buffer; wrapped: string; uses: number } | null = null;

  async init(): Promise<void> {
    this.addr = (process.env.VAULT_ADDR || "http://127.0.0.1:8200").replace(/\/+$/, "");
    this.token = process.env.VAULT_TOKEN || "";
    if (!this.token) throw new Error("VAULT_TOKEN is required for the vault-transit backend");
    this.key = process.env.VAULT_TRANSIT_KEY || "bench";
    this.mode = process.env.VAULT_TRANSIT_MODE === "envelope" ? "envelope" : "direct";
    this.derived = process.env.VAULT_TRANSIT_DERIVED === "true";
    this.maxUses = Math.max(1, Number(process.env.VAULT_DATA_KEY_MAX_USES ?? 1));
  }

  /** POST to the Vault HTTP API and return `data`, throwing on transport or Vault error. */
  private async vault(path: string, body: unknown): Promise<Record<string, unknown>> {
    const res = await fetch(`${this.addr}/v1/${path}`, {
      method: "POST",
      headers: { "X-Vault-Token": this.token, "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const json = (await res.json().catch(() => ({}))) as { data?: Record<string, unknown>; errors?: unknown };
    if (!res.ok) throw new Error(`vault ${path} failed (${res.status}): ${JSON.stringify(json.errors ?? json)}`);
    return json.data ?? {};
  }

  encryptBatch(input: PlainRecord[], stats?: OpStats): Promise<EncRecord[]> {
    return this.mode === "envelope" ? this.encryptEnvelope(input, stats) : this.encryptDirect(input, stats);
  }

  decryptBatch(input: EncRecord[], stats?: OpStats): Promise<PlainRecord[]> {
    return this.mode === "envelope" ? this.decryptEnvelope(input, stats) : this.decryptDirect(input, stats);
  }

  // --- direct mode: plaintext -> Vault -> ciphertext, one batch round-trip ---

  private async encryptDirect(input: PlainRecord[], stats?: OpStats): Promise<EncRecord[]> {
    const ctxs = input.map(() => (this.derived ? randomBytes(16).toString("base64") : undefined));
    const items = input.flatMap((r, i) =>
      FIELDS.map((f) => ({
        plaintext: Buffer.from(r[f], "utf-8").toString("base64"),
        ...(ctxs[i] ? { context: ctxs[i] } : {}),
      })),
    );
    const data = await this.vault(`transit/encrypt/${this.key}`, { batch_input: items });
    if (stats) stats.kmsCalls += 1; // one batched round-trip, regardless of batch size
    const results = batchResults<{ ciphertext: string }>(data);
    return input.map((_, i) => {
      const rec = {} as EncRecord;
      FIELDS.forEach((f, j) => {
        const cell: DirectCell = { ct: results[i * FIELDS.length + j].ciphertext };
        if (ctxs[i]) cell.ctx = ctxs[i];
        rec[f] = JSON.stringify(cell);
      });
      return rec;
    });
  }

  private async decryptDirect(input: EncRecord[], stats?: OpStats): Promise<PlainRecord[]> {
    const parsed = input.map((r) => Object.fromEntries(FIELDS.map((f) => [f, JSON.parse(r[f]) as DirectCell])) as Record<Field, DirectCell>);
    const items = parsed.flatMap((p) =>
      FIELDS.map((f) => ({ ciphertext: p[f].ct, ...(p[f].ctx ? { context: p[f].ctx } : {}) })),
    );
    const data = await this.vault(`transit/decrypt/${this.key}`, { batch_input: items });
    if (stats) stats.kmsCalls += 1; // one batched round-trip
    const results = batchResults<{ plaintext: string }>(data);
    return parsed.map((_, i) => {
      const rec = {} as PlainRecord;
      FIELDS.forEach((f, j) => {
        rec[f] = Buffer.from(results[i * FIELDS.length + j].plaintext, "base64").toString("utf-8");
      });
      return rec;
    });
  }

  // --- envelope mode: Vault `datakey` + local AES-256-GCM (plaintext stays client-side) ---

  private async getWriteKey(stats?: OpStats) {
    if (this.writeKey && this.writeKey.uses < this.maxUses) {
      this.writeKey.uses += 1;
      return this.writeKey;
    }
    const data = await this.vault(`transit/datakey/plaintext/${this.key}`, { bits: 256 });
    if (stats) stats.kmsCalls += 1; // one `datakey` round-trip per data key (no batch API)
    const key = {
      plaintext: Buffer.from(data.plaintext as string, "base64"),
      wrapped: data.ciphertext as string,
      uses: 1,
    };
    this.writeKey = key;
    return key;
  }

  private encryptOneWith(dek: { plaintext: Buffer; wrapped: string }, plaintext: string): string {
    const iv = randomBytes(12);
    const cipher = createCipheriv(ALGO, dek.plaintext, iv);
    const ct = Buffer.concat([cipher.update(plaintext, "utf-8"), cipher.final()]);
    const cell: EnvelopeCell = {
      edk: dek.wrapped,
      iv: iv.toString("base64"),
      tag: cipher.getAuthTag().toString("base64"),
      ct: ct.toString("base64"),
    };
    return JSON.stringify(cell);
  }

  private async encryptEnvelope(input: PlainRecord[], stats?: OpStats): Promise<EncRecord[]> {
    if (this.maxUses > 1) {
      // SEQUENTIAL so the cached data key is actually reused across values.
      const out: EncRecord[] = [];
      for (const r of input) {
        const rec = {} as EncRecord;
        for (const f of FIELDS) rec[f] = this.encryptOneWith(await this.getWriteKey(stats), r[f]);
        out.push(rec);
      }
      return out;
    }
    // per-value (maxUses=1): fresh data key per value. `datakey` has no batch, so
    // this is one Vault round-trip per value — the cost of per-record keys on write.
    return Promise.all(
      input.map(async (r) => {
        const rec = {} as EncRecord;
        for (const f of FIELDS) rec[f] = this.encryptOneWith(await this.getWriteKey(stats), r[f]);
        return rec;
      }),
    );
  }

  private async decryptEnvelope(input: EncRecord[], stats?: OpStats): Promise<PlainRecord[]> {
    const parsed = input.map((r) => Object.fromEntries(FIELDS.map((f) => [f, JSON.parse(r[f]) as EnvelopeCell])) as Record<Field, EnvelopeCell>);
    const distinct = [...new Set(parsed.flatMap((p) => FIELDS.map((f) => p[f].edk)))];
    // Unwrap all DISTINCT data keys in ONE batched `decrypt` round-trip. This is
    // where Vault diverges from AWS KMS envelope: a scattered read references many
    // distinct keys but still costs a single round-trip, not one per record.
    const dekFor = new Map<string, Buffer>();
    if (distinct.length) {
      const data = await this.vault(`transit/decrypt/${this.key}`, {
        batch_input: distinct.map((edk) => ({ ciphertext: edk })),
      });
      if (stats) stats.kmsCalls += 1;
      const results = batchResults<{ plaintext: string }>(data);
      distinct.forEach((edk, i) => dekFor.set(edk, Buffer.from(results[i].plaintext, "base64")));
    }
    // AES-decrypt every value locally with its (now in-memory) data key.
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

/** Pull `batch_results` from a Transit response, surfacing any per-item error. */
function batchResults<T>(data: Record<string, unknown>): T[] {
  const results = (data.batch_results ?? []) as (T & { error?: string })[];
  for (const r of results) if (r.error) throw new Error(`vault batch item error: ${r.error}`);
  return results;
}

export function createVaultTransitBackend(): EncryptionBackend {
  return new VaultTransitBackend();
}
