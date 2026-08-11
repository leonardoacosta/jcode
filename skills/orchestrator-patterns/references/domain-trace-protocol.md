# Domain Trace Protocol

> Trace a business domain across all monorepo layers. Detect gaps, misplacements, and naming misalignments.

## Step 1: Discover Domains

The canonical domain list comes from DB schema subdirectories. Each subdirectory = one domain.

```bash
ls packages/db/src/schemas/
```

**Known domains (15):** admin, auth, badges, communications, content, core, marketing,
notifications, payments, security, shared, sponsorships, vendors, venues, volunteers

This is the master list. Any domain that exists elsewhere but NOT here is an orphan. Any
subdirectory here that has no downstream presence is a stub.

## Step 2: Trace Through Layers

For each domain, check presence in these layers (order = data flow direction):

| # | Layer | Path Pattern | What to Find |
|---|-------|-------------|--------------|
| 1 | DB schemas | `packages/db/src/schemas/{domain}/` | Tables, enums, type definitions |
| 2 | DB relations | `packages/db/src/relations/{domain}-relations.ts` | Drizzle relation definitions |
| 3 | DB domain barrel | `packages/db/src/domains/{domain}.ts` | Re-export aggregating schema + relations |
| 4 | Validators | `packages/validators/src/{domain}/` | Zod input schemas for tRPC |
| 5 | API services | `packages/api/src/services/{domain}/` | Business logic |
| 6 | API router | `packages/api/src/router/{domain}/` or `router/admin/{domain}*.ts` | tRPC procedures |
| 7 | Auth permissions | `packages/auth/src/permissions.ts` | Permission resource matching domain |
| 8 | UI components | `apps/nextjs/src/components/{domain}/` | React components |
| 9 | Pages | `apps/nextjs/src/app/*/{domain}/` | Route pages (check all route groups) |
| 10 | Hooks | `apps/nextjs/src/hooks/use-{domain}*.ts` | Domain-specific React hooks |
| 11 | E2E tests | `packages/e2e/tests/**/{domain}*.spec.ts` | End-to-end test coverage |

Not every domain needs all 11 layers. `shared` and `core` are infrastructure domains — they
typically lack routes, components, and E2E tests. Judge gaps against domain purpose.

## Step 3: Gap Detection Rules

### Structural Gaps

| Condition | Severity | Type |
|-----------|----------|------|
| Domain in schemas but NOT in validators or services | HIGH | `missing_layer` |
| Domain in schemas but no `{domain}-relations.ts` | HIGH | `missing_relations` |
| Domain in schemas but no `domains/{domain}.ts` barrel | LOW | `missing_barrel` |
| Domain in services but NOT in schemas | MEDIUM | `orphan_service` |
| Domain in components but NOT in services | MEDIUM | `orphan_ui` |

### Naming Misalignments

| Condition | Severity | Type |
|-----------|----------|------|
| DB uses `X` but API service uses `X-Y` | MEDIUM | `naming` |
| DB uses singular, component dir uses plural (or vice versa) | LOW | `naming` |
| Relations file has typo (e.g., `badgeing` vs `badge`) | LOW | `naming` |

### Domain Leakage

| Condition | Severity | Type |
|-----------|----------|------|
| Schema files for domain A inside `schemas/{domain_B}/` | HIGH | `misplaced` |
| Service file for domain A inside `services/{domain_B}/` | HIGH | `misplaced` |
| Component for domain A inside `components/{domain_B}/` | MEDIUM | `misplaced` |

### Orphans

| Condition | Severity | Type |
|-----------|----------|------|
| File matches domain name but lives outside domain directory | LOW | `orphan` |
| Route group references domain with no matching component dir | MEDIUM | `orphan` |

## Step 4: Output Format

Auditor agents MUST output one JSON object per domain:

```json
{
  "domain": "badges",
  "layers": {
    "db_schemas": ["badges.ts", "badge-purchase.ts"],
    "db_relations": ["badgeing-relations.ts"],
    "db_domains": ["badges.ts"],
    "validators": ["enums.ts", "purchase.ts"],
    "api_services": ["badge/", "badge-cache/"],
    "api_router": ["badges/", "badgeAdmin.ts"],
    "auth": ["badges: 11 actions"],
    "nextjs_components": ["badges/ (26 files)"],
    "nextjs_pages": ["(staff)/badges/", "(pages)/badges/"],
    "nextjs_hooks": ["use-badge-data.ts"],
    "e2e": ["badge-purchase-journey.spec.ts"]
  },
  "gaps": [
    {"layer": "db_relations", "type": "naming", "severity": "LOW", "details": "badgeing-relations.ts has typo"},
    {"layer": "db_schemas", "type": "misplaced", "severity": "HIGH", "details": "cosplay-*.ts belongs in schemas/cosplay/"}
  ],
  "file_count": 160,
  "healthy_count": 148
}
```

**Field rules:**
- `layers`: List files/directories found at each layer. Use `[]` for absent layers.
- `gaps`: Every detected issue. Empty array `[]` if domain is clean.
- `file_count`: Total files touched by this domain across all layers.
- `healthy_count`: Files with no detected issues.

## Step 5: Cross-Domain Checks

Run these AFTER all individual domain traces complete.

### Coupling Detection

```bash
# Find files importing from multiple domain schemas
for f in $(find packages/api/src/services -name "*.ts"); do
  domains=$(grep -oP 'schemas/\K[^/]+' "$f" | sort -u | wc -l)
  [[ $domains -gt 1 ]] && echo "$f imports $domains domains"
done
```

### Leaky Barrel Exports

```bash
# Check if a domain barrel imports from another domain's schema dir
for barrel in packages/db/src/domains/*.ts; do
  domain=$(basename "$barrel" .ts)
  foreign=$(grep -oP 'schemas/\K[^/]+' "$barrel" | grep -v "$domain" || true)
  [[ -n "$foreign" ]] && echo "$barrel re-exports from: $foreign"
done
```

### Naming Alignment Table

Build this table per domain and flag mismatches:

| Domain (DB) | API Service Dir | Component Dir | Route Segment | Aligned? |
|-------------|-----------------|---------------|---------------|----------|
| badges | badge/ | badges/ | badges/ | NO (singular vs plural in service) |
| vendors | vendor/ | vendors/ | vendors/ | NO |
| payments | payments/ | payments/ | payments/ | YES |

## Detection Commands

### Layer Checks (per domain)

```bash
DOMAIN="badges"

ls packages/db/src/schemas/${DOMAIN}/ 2>/dev/null              # schemas
ls packages/db/src/relations/${DOMAIN}-relations.ts 2>/dev/null # relations
ls packages/db/src/domains/${DOMAIN}.ts 2>/dev/null             # barrel
ls packages/validators/src/${DOMAIN}/ 2>/dev/null               # validators
ls packages/api/src/services/${DOMAIN}/ 2>/dev/null             # services
ls packages/api/src/router/${DOMAIN}/ 2>/dev/null               # router
grep -c "${DOMAIN}" packages/auth/src/permissions.ts 2>/dev/null # auth
ls apps/nextjs/src/components/${DOMAIN}/ 2>/dev/null            # components
find apps/nextjs/src/app -type d -name "${DOMAIN}" 2>/dev/null  # pages
ls apps/nextjs/src/hooks/use-${DOMAIN}*.ts 2>/dev/null          # hooks
find packages/e2e/tests -name "${DOMAIN}*.spec.ts" 2>/dev/null  # e2e
```

### Misplacement Detection

```bash
DOMAIN="cosplay"
for dir in packages/db/src/schemas/*/; do
  dir_domain=$(basename "$dir")
  [[ "$dir_domain" == "$DOMAIN" ]] && continue
  matches=$(grep -rl "$DOMAIN" "$dir" --include="*.ts" 2>/dev/null)
  [[ -n "$matches" ]] && echo "Possible misplacement in $dir_domain: $matches"
done
```

### Full Trace (all domains)

```bash
for domain in $(ls packages/db/src/schemas/); do
  echo "=== $domain ==="
  for layer in \
    "packages/db/src/schemas/${domain}/" \
    "packages/db/src/relations/${domain}-relations.ts" \
    "packages/db/src/domains/${domain}.ts" \
    "packages/validators/src/${domain}/" \
    "packages/api/src/services/${domain}/" \
    "packages/api/src/router/${domain}/"; do
    [[ -e "$layer" ]] && echo "  FOUND: $layer" || echo "  MISSING: $layer"
  done
done
```

## Anti-Patterns

| Pattern | Problem | Fix |
|---------|---------|-----|
| Trace only schemas and services | Misses UI orphans, missing validators | Trace all 11 layers |
| Skip infrastructure domains (shared, core, auth) | False positives on missing UI layers | Judge gaps against domain purpose |
| Report naming differences without checking usage | Singular/plural may be intentional | Check if both forms are used inconsistently |
| Run cross-domain checks per-domain | Cannot detect coupling without full picture | Run cross-domain checks after all traces |
