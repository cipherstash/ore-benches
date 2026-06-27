import type { PlainRecord } from "./encryption/types";

// Generate unique synthetic records server-side, so the load profile only sends
// a `count` (not 60 plaintext values per request) and we isolate crypto cost
// from HTTP body size.
let seq = 0;

export function makeRecords(count: number): PlainRecord[] {
  return Array.from({ length: count }, () => {
    seq += 1;
    const u = `${Date.now()}-${seq}-${Math.floor(Math.random() * 1e9)}`;
    return {
      email: `user-${u}@example.com`,
      name: `User ${u}`,
      phone: `+1${1000000000 + Math.floor(Math.random() * 8999999999)}`,
    };
  });
}
