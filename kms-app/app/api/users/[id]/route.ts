import { NextResponse } from "next/server";
import { pool } from "../../../../lib/db";
import { getBackend } from "../../../../lib/encryption";

/**
 * GET /api/users/:id — read a user, decrypting email + name. This is the
 * "read" path; it exercises the decrypt + key-fetch cost of each backend.
 */
export async function GET(
  _request: Request,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;

  const { rows } = await pool.query(
    `SELECT id, email_encrypted, name_encrypted FROM users WHERE id = $1`,
    [id],
  );
  if (rows.length === 0) {
    return NextResponse.json({ error: "not found" }, { status: 404 });
  }

  const backend = await getBackend();
  const [email, name] = await Promise.all([
    backend.decrypt(rows[0].email_encrypted, "email"),
    backend.decrypt(rows[0].name_encrypted, "name"),
  ]);

  return NextResponse.json({ id: rows[0].id, email, name });
}
