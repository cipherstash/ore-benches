import { NextResponse } from "next/server";
import { pool } from "../../../../lib/db";
import { getBackend } from "../../../../lib/encryption";
import type { OpStats } from "../../../../lib/encryption/types";

/**
 * GET /api/records/query?limit=20&scatter=false
 *
 * Read `limit` existing rows and decrypt their limit×3 fields.
 *
 *   - scatter=false (default): a contiguous id window — records that were
 *     inserted together. Under data-key REUSE these share a data key.
 *   - scatter=true: `limit` records by RANDOM ids across the whole table —
 *     a realistic retrieval pattern that has nothing to do with insert order,
 *     so the records reference ~`limit` DISTINCT data keys.
 *
 * Returns `kmsCalls` (key-service round-trips this request made). For the
 * envelope backend that equals the distinct data keys in the result — the
 * crux of the experiment. (Run with ONE backend's rows in the table so ids are
 * contiguous; restart the server between seed and query so the range is fresh.)
 */
let idRange: { min: number; max: number } | null = null;

async function getRange(backendName: string) {
  if (idRange) return idRange;
  const r = await pool.query(
    `SELECT min(id)::bigint AS lo, max(id)::bigint AS hi FROM records WHERE backend = $1`,
    [backendName],
  );
  const lo = Number(r.rows[0]?.lo);
  const hi = Number(r.rows[0]?.hi);
  if (!lo || !hi) return null;
  idRange = { min: lo, max: hi };
  return idRange;
}

export async function GET(request: Request) {
  const sp = new URL(request.url).searchParams;
  const limit = clamp(Number(sp.get("limit") ?? 20), 1, 10000);
  const scatter = sp.get("scatter") === "true";

  try {
    const backend = await getBackend();
    const range = await getRange(backend.name);
    if (!range) {
      return NextResponse.json(
        { error: `no '${backend.name}' rows — seed via the insert benchmark first` },
        { status: 409 },
      );
    }

    const cols = "email_encrypted, name_encrypted, phone_encrypted";
    const idSpan = range.max - range.min + 1;
    let rows;
    if (scatter) {
      const ids = new Set<number>();
      for (let g = 0; ids.size < limit && g < limit * 30; g++) {
        ids.add(range.min + Math.floor(Math.random() * idSpan));
      }
      ({ rows } = await pool.query(
        `SELECT ${cols} FROM records WHERE backend = $1 AND id = ANY($2)`,
        [backend.name, [...ids]],
      ));
    } else {
      const start = range.min + Math.floor(Math.random() * Math.max(1, idSpan - limit + 1));
      ({ rows } = await pool.query(
        `SELECT ${cols} FROM records WHERE backend = $1 AND id >= $2 ORDER BY id LIMIT $3`,
        [backend.name, start, limit],
      ));
    }

    const stats: OpStats = { kmsCalls: 0 };
    const decrypted = await backend.decryptBatch(
      rows.map((r) => ({ email: r.email_encrypted, name: r.name_encrypted, phone: r.phone_encrypted })),
      stats,
    );
    return NextResponse.json({
      count: decrypted.length,
      kmsCalls: stats.kmsCalls,
      pattern: scatter ? "scattered" : "sequential",
    });
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
