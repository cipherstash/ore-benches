import type { EncryptionBackend } from "./types";
import { createZeroKmsBackend } from "./zerokms";
import { createAwsKmsBackend } from "./aws-kms";

export type { EncryptionBackend, Field } from "./types";

let backendPromise: Promise<EncryptionBackend> | null = null;

/**
 * Resolve the encryption backend selected by ENCRYPTION_BACKEND.
 * Cached so the underlying client (and its connection pool / schema
 * registration) is constructed once per server process.
 */
export function getBackend(): Promise<EncryptionBackend> {
  if (backendPromise) return backendPromise;

  const selected = process.env.ENCRYPTION_BACKEND;
  const backend =
    selected === "zerokms"
      ? createZeroKmsBackend()
      : selected === "aws-kms"
        ? createAwsKmsBackend()
        : null;

  if (!backend) {
    throw new Error(
      `ENCRYPTION_BACKEND must be 'zerokms' or 'aws-kms' (got: ${selected ?? "unset"})`,
    );
  }

  backendPromise = backend.init().then(() => backend);
  return backendPromise;
}
