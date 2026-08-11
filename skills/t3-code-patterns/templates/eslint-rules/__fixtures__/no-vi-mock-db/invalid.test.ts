// Fixture: invalid — each vi.mock against a @<ws>/db package should fire.
// Note: lives under __tests__ but NOT under __tests__/integration/, so the
// rule still applies (escape hatch is only the integration subdir).
import { vi } from "vitest";

// expect: report
vi.mock("@acme/db");

// expect: report — subpath of the db package
vi.mock("@acme/db/client", () => ({ db: {} }));

// expect: report — template literal form
vi.mock(`@storefront/db/schema`);

// expect: report — different workspace prefix, still matches
vi.mock("@backoffice/db", () => ({}));
