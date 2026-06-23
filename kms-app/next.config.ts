import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // The CipherStash SDK and AWS SDK are server-only; keep them external to the
  // server bundle so native/optional deps resolve at runtime.
  serverExternalPackages: ["@cipherstash/stack", "@aws-sdk/client-kms", "pg"],
};

export default nextConfig;
