export default function Home() {
  return (
    <main style={{ fontFamily: "system-ui", padding: "2rem", maxWidth: 640 }}>
      <h1>KMS comparison harness</h1>
      <p>
        A thin CRUD app used to load-test field encryption with swappable
        backends. The active backend is set per server process via{" "}
        <code>ENCRYPTION_BACKEND</code> (<code>zerokms</code> |{" "}
        <code>aws-kms</code>).
      </p>
      <ul>
        <li>
          <code>POST /api/users</code> — create + encrypt
        </li>
        <li>
          <code>GET /api/users/:id</code> — read + decrypt
        </li>
        <li>
          <code>GET /api/health</code> — DB + backend readiness
        </li>
      </ul>
      <p>See README.md to run a load test.</p>
    </main>
  );
}
