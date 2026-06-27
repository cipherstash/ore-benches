import { NextResponse } from "next/server";
import { pool } from "../../../../lib/db";
import { getBackend } from "../../../../lib/encryption";
import { FIELDS } from "../../../../lib/encryption/types";
import { makeRecords } from "../../../../lib/records";

/**
 * POST /api/records/insert  { count }
 *
 * The write benchmark: generate `count` records, bulk-encrypt all
 * count×3 fields, and multi-row insert. ZeroKMS does the encrypt in one round
 * trip; AWS does count×3 KMS calls.
 */
export async function POST(request: Request) {
  const body = await request.json().catch(() => null);
  const count = clamp(Number(body?.count ?? 20), 1, 10000);

  try {
    const backend = await getBackend();
    const encrypted = await backend.encryptBatch(makeRecords(count));

    // Build a single multi-row INSERT. Params: $1 = backend, then 3 per record.
    const cols = FIELDS.length;
    const rowsSql = encrypted
      .map((_, i) => `($1, ${FIELDS.map((__, j) => `$${i * cols + j + 2}`).join(", ")})`)
      .join(", ");
    const params: string[] = [backend.name, ...encrypted.flatMap((e) => FIELDS.map((f) => e[f]))];

    const { rowCount } = await pool.query(
      `INSERT INTO records (backend, email_encrypted, name_encrypted, phone_encrypted)
       VALUES ${rowsSql}`,
      params,
    );
    return NextResponse.json({ inserted: rowCount }, { status: 201 });
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
