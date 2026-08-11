// Fixture: env.ts is the sanctioned boundary — direct process.env reads here are
// EXEMPT (this is where the createEnv schema legitimately consumes process.env).
// Expect 0 reports despite the raw reads below.
import { createEnv } from "@t3-oss/env-nextjs";

export const env = createEnv({
  server: {},
  runtimeEnv: {
    POSTGRES_URL: process.env.POSTGRES_URL,
    NODE_ENV: process.env.NODE_ENV,
    STRIPE_SECRET_KEY: process.env.STRIPE_SECRET_KEY,
  },
});
