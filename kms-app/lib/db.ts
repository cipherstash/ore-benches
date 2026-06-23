import { Pool } from "pg";

/**
 * Shared Postgres pool. Defaults to the benches native cluster
 * (postgres://postgres:postgres@localhost:5400/postgres) — see the repo
 * root mise.toml `postgres` task to start it.
 */
const globalForPool = globalThis as unknown as { pool?: Pool };

export const pool =
  globalForPool.pool ??
  new Pool({
    connectionString:
      process.env.DATABASE_URL ??
      "postgres://postgres:postgres@localhost:5400/postgres",
    max: Number(process.env.PG_POOL_MAX ?? 20),
  });

if (process.env.NODE_ENV !== "production") globalForPool.pool = pool;
