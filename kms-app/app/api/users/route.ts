import { NextResponse } from "next/server";
import { pool } from "../../../lib/db";
import { getBackend } from "../../../lib/encryption";

/**
 * POST /api/users — create a user, encrypting email + name with the selected
 * backend before storing. This is the "write" path Artillery hammers.
 */
export async function POST(request: Request) {
  const body = await request.json().catch(() => null);
  if (!body?.email || !body?.name) {
    return NextResponse.json(
      { error: "email and name are required" },
      { status: 400 },
    );
  }

  const backend = await getBackend();
  const [emailEncrypted, nameEncrypted] = await Promise.all([
    backend.encrypt(String(body.email), "email"),
    backend.encrypt(String(body.name), "name"),
  ]);

  const { rows } = await pool.query(
    `INSERT INTO users (backend, email_encrypted, name_encrypted)
     VALUES ($1, $2, $3) RETURNING id`,
    [backend.name, emailEncrypted, nameEncrypted],
  );

  return NextResponse.json({ id: rows[0].id }, { status: 201 });
}
