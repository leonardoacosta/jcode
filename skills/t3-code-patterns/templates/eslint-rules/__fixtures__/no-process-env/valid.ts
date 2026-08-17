// Fixture: valid — none of these should fire. All env access flows through the
// validated `env` object from @t3-oss/env; there is no direct `process.env` here.
import { env } from "~/env";

export function dbUrl() {
  return env.POSTGRES_URL;
}

export function isProd() {
  return env.NODE_ENV === "production";
}

export function stripeKey() {
  const { STRIPE_SECRET_KEY } = env;
  return STRIPE_SECRET_KEY;
}

export function unrelated(ctx: { env: Record<string, string> }) {
  // A different object named `env` — only `process.env` is the target.
  return ctx.env.SOMETHING;
}
