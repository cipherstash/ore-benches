export default function Home() {
  return (
    <main style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: 680 }}>
      <h1>KMS comparison harness</h1>
      <p>
        Load-tests batch field encryption with swappable backends. The active
        backend is set per server process via <code>ENCRYPTION_BACKEND</code> (
        <code>zerokms</code> | <code>aws-kms</code> |{" "}
        <code>aws-kms-envelope</code>). Each request encrypts/decrypts many
        values at once — that batch amortization is the whole point.
      </p>
      <ul>
        <li>
          <code>POST /api/records/insert</code> {"{ count }"} — bulk encrypt +
          insert
        </li>
        <li>
          <code>GET /api/records/query?limit=N</code> — bulk read + decrypt
        </li>
        <li>
          <code>GET /api/health</code> — DB + backend readiness
        </li>
      </ul>
      <p>See README.md to run the insert and query benchmarks.</p>
    </main>
  );
}
