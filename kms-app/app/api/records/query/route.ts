import { NextResponse } from "next/server";
import { pool } from "../../../../lib/db";
import { getBackend } from "../../../../lib/encryption";

/**
 * GET /api/records/query?limit=20
 *
 * The read benchmark: read a random window of `limit` existing rows (written by
 * the same backend) and bulk-decrypt all limit×3 fields. ZeroKMS decrypts in
 * one round trip; AWS does limit×3 KMS calls. Seed the table first by running
 * the insert benchmark against the same backend.
 *
 * Returns only a count — the decrypted PII never leaves the process, and we
 * avoid serialization cost polluting the latency measurement.
 */
let idRange: { min: number; max: number } | null = null;

export async function GET(request: Request) {
  const limit = clamp(Number(new URL(request.url).searchParams.get("limit") ?? 20), 1, 10000);

  try {
    const backend = await getBackend();

    // Cache the id range for this backend once per process (cheap, indexed).
    if (!idRange) {
      const r = await pool.query(
        `SELECT min(id)::bigint AS lo, max(id)::bigint AS hi FROM records WHERE backend = $1`,
        [backend.name],
      );
      const lo = Number(r.rows[0]?.lo);
      const hi = Number(r.rows[0]?.hi);
      if (!lo || !hi) {
        return NextResponse.json(
          { error: `no '${backend.name}' rows — run the insert benchmark to seed first` },
          { status: 409 },
        );
      }
      idRange = { min: lo, max: hi };
    }

    const span = Math.max(1, idRange.max - idRange.min - limit + 1);
    const start = idRange.min + Math.floor(Math.random() * span);
    const { rows } = await pool.query(
      `SELECT email_encrypted, name_encrypted, phone_encrypted
         FROM records WHERE backend = $1 AND id >= $2 ORDER BY id LIMIT $3`,
      [backend.name, start, limit],
    );

    const decrypted = await backend.decryptBatch(
      rows.map((r) => ({
        email: r.email_encrypted,
        name: r.name_encrypted,
        phone: r.phone_encrypted,
      })),
    );
    return NextResponse.json({ count: decrypted.length });
  } catch (error) {
    return NextResponse.json(
      { error: error instanceof Error ? error.message : String(error) },
      { status: 500 },
    );
  }
}

function clamp(n: number, lo: number, hi: number): number {
  return Math.min(Math.max(Number.isFinite(n) ? n : lo, lo), hi);
}
