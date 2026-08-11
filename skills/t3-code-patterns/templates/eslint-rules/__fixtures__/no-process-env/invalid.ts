// Fixture: invalid — every direct `process.env.X` read below should trigger
// `no-process-env`. Mirrors the ~780 bypassed reads found in the 2026-07-06
// fleet survey (storefront 270, acme 250, operations 185, backoffice 35, api-app 30, portal 10).

export function dbUrl() {
  // expect: report on process.env.POSTGRES_URL
  return process.env.POSTGRES_URL;
}

export function isProd() {
  // expect: report on process.env.NODE_ENV
  return process.env.NODE_ENV === "production";
}

export function stripeKey() {
  // expect: report — assignment target is irrelevant; the read is the smell
  const key = process.env.STRIPE_SECRET_KEY;
  return key;
}

export function computed(name: string) {
  // expect: report — computed access is even more opaque (reports as <computed>)
  return process.env[name];
}
