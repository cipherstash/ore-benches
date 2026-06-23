import { NextResponse } from "next/server";
import { pool } from "../../../lib/db";
import { getBackend } from "../../../lib/encryption";

/**
 * GET /api/health — verifies the DB connection and that the selected
 * encryption backend initializes. Run this before a load test to fail fast
 * on missing credentials rather than mid-run.
 */
export async function GET() {
  try {
    await pool.query("SELECT 1");
    const backend = await getBackend();
    return NextResponse.json({ ok: true, backend: backend.name });
  } catch (error) {
    return NextResponse.json(
      { ok: false, error: error instanceof Error ? error.message : String(error) },
      { status: 503 },
    );
  }
}
