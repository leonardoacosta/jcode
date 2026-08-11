
# Integration Styles: POM vs Fixture

## Page Object Model (POM)

The POM pattern encapsulates login page locators and actions in a class. Test files
instantiate the page object and call methods like `loginWithCredentials()`.

**Example location:** `packages/e2e/src/tests/auth/pages/login-page.ts`

Key characteristics:
- Class-based: `LoginPage` owns all locators (email, password, submit, error, etc.)
- Locators use fallback chains: `[data-testid="login-email"], input[type="email"]`
- Methods return `Promise<void>` -- callers assert on page state after
- `waitForLoginResult()` races URL change vs error message visibility
- Constructor accepts `baseUrl` for environment flexibility

**When to use POM:**
- Complex login pages with OAuth buttons, remember-me, rate limiting
- Tests that need to assert on login page UI state (error messages, validation)
- Projects where multiple test files share the same login page interactions

## Fixture Pattern

The fixture pattern extends Playwright's `test.extend()` to provide pre-authenticated
pages as test fixtures. Each role gets its own fixture, backed by the canonical persona
catalog and `setup`-generated `.auth/<persona>.json` state.

**Example location:** `packages/e2e/fixtures/auth.fixture.ts`

Key characteristics:
- Extends `base.test` with typed fixtures: `platformAdminPage`, `agencyAdminPage`, etc.
- Each fixture creates a fresh `BrowserContext` and `Page` from the requested persona's storage state
- Persona definitions own role, credential env keys/fallbacks, and storage-state paths
- Tests never silently substitute a different or more privileged persona
- Context is closed in fixture teardown (after `use()`)

**When to use fixtures:**
- Multi-role apps where tests need pre-authenticated pages per role
- Tests that focus on post-login functionality, not the login flow itself
- Projects that want zero auth boilerplate in test bodies

## Provider Effects Are Orthogonal to POM vs Fixture

POM and fixture patterns decide how the browser reaches an authenticated state. They do not decide
how terminal provider effects are substituted. Verification email, invitations, magic links, and
password reset flows use the same application-owned `EmailDelivery` interface in both styles:

| Layer | E2E behavior |
| --- | --- |
| Browser route and form | Real Playwright interaction |
| Auth service and token persistence | Real application behavior |
| Terminal email delivery | Deterministic run-scoped email adapter |
| Link/token lookup | Exact safe receipt from the deterministic mailbox store |
| Resend integration | Separate production-adapter integration lane |

Do not fulfill the auth endpoint with `page.route` and do not poll a shared provider inbox. Both
patterns weaken isolation: the first bypasses server behavior, while the second creates cross-run
state and irreversible external delivery. Both styles remain bound to setup-project generation per
CI run, isolated role contexts, and run-scoped `.auth/<persona>.json` state.
