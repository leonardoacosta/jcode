// Fixture: valid — none of these should fire.
import { vi } from "vitest";

// Mocking other packages is allowed; only @<ws>/db is forbidden.
vi.mock("@acme/auth");
vi.mock("stripe");
vi.mock("@acme/db-utils");      // not the db package — different name
vi.mock("@acme/database");      // also unrelated; only `/db` boundary matches

// Different callees — not `vi.mock`.
vi.fn();
vi.spyOn({}, "method");

// String similar to the db package but not at start — should not match.
const someName = "x@acme/db";
void someName;
