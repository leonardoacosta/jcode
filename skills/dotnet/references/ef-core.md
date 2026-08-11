# EF Core

DbContext registration, query patterns, migrations. Targets EF Core 8/9 on .NET 8/9/10.

## DbContext registration

Register the context in `Program.cs`. Scoped lifetime is the default and correct for a
per-request web app — never register a `DbContext` as singleton.

```csharp
builder.Services.AddDbContext<DocumentContext>(options =>
    options.UseSqlServer(
        builder.Configuration.GetConnectionString("Documents"),
        sql => sql.EnableRetryOnFailure()));   // transient-fault resilience
```

For pooled contexts (high-throughput APIs, no per-instance state):

```csharp
builder.Services.AddDbContextPool<DocumentContext>(options => ...);
```

Do NOT capture a pooled context field that survives the request — pooling reuses instances.

## Query patterns

| Rule | Why |
| --- | --- |
| Propagate `CancellationToken` to every async EF call | Client-disconnect cancels the DB round-trip |
| `AsNoTracking()` on read-only queries | Skips change-tracker snapshot — faster, less memory |
| Project with `.Select(x => new Dto(...))` | Avoids over-fetching whole entities |
| Filter soft-deletes (`DeletedOn == null`) explicitly, or via a global query filter | Consistency across every read |
| `AsSplitQuery()` for multiple collection includes | Avoids the cartesian-explosion row blowup of a single JOIN |

```csharp
public async Task<DocumentDto?> GetAsync(Guid id, CancellationToken ct)
{
    return await _db.Documents
        .AsNoTracking()
        .Where(d => d.Id == id && d.DeletedOn == null)
        .Select(d => new DocumentDto(d.Id, d.Title, d.Status))
        .FirstOrDefaultAsync(ct);
}
```

Split query for multiple collections (single-query mode would fan out rows N*M):

```csharp
var order = await _db.Orders
    .AsNoTracking()
    .AsSplitQuery()
    .Include(o => o.Lines)
    .Include(o => o.Shipments)
    .FirstOrDefaultAsync(o => o.Id == id, ct);
```

Global soft-delete filter (declare once in `OnModelCreating`, then reads are filter-free):

```csharp
modelBuilder.Entity<Document>().HasQueryFilter(d => d.DeletedOn == null);
// Bypass deliberately when you need soft-deleted rows:
_db.Documents.IgnoreQueryFilters().Where(...)
```

## Anti-patterns

| Anti-pattern | Replace with |
| --- | --- |
| `.Result` / `.Wait()` on an EF task | `await` |
| Tracking query for a read | `.AsNoTracking()` |
| `.ToList()` then LINQ-to-objects filter | Push the `.Where` into the DB query |
| N+1 (`foreach` with a per-row query) | `.Include(...)` or a projection join |
| Fetching the entity to read one column | `.Select(x => x.Column)` |

## Migrations workflow

Migration-based only. Generate, review the generated C#, commit, then apply on deploy.

```bash
# Create a migration after changing entities / DbContext config
dotnet ef migrations add AddDocumentStatus --project api/DC.Infrastructure --startup-project api/DC.Api

# Apply to a local/dev database
dotnet ef database update --project api/DC.Infrastructure --startup-project api/DC.Api

# Roll back to a prior migration
dotnet ef database update PreviousMigrationName --project api/DC.Infrastructure --startup-project api/DC.Api

# Produce an idempotent SQL script for a controlled deploy (preferred for prod)
dotnet ef migrations script --idempotent --output migrate.sql --project api/DC.Infrastructure --startup-project api/DC.Api
```

Always read the generated `Migrations/*.cs` before committing — EF can generate a destructive
column drop/recreate for a change you intended to be additive.

## Migration-on-deploy caution

`context.Database.Migrate()` at app startup is convenient but has real foot-guns:

- Every replica races to apply the same migration on rollout — take a lock or run migration in a
  single pre-deploy job, not in `Program.cs` on every instance.
- A migration failure in the startup path can crash-loop the whole app.
- It couples schema change to deploy timing, so you cannot roll app + schema independently.

Prefer a **one-shot pre-deploy migration job** (`dotnet ef database update` or the idempotent SQL
script run once by the pipeline) over in-process `Migrate()`. If you must migrate in-process, gate
it to a single instance and never swallow the failure — let it surface, do not
`catch { /* ignore */ }` a failed migration (the "ignore DB sync errors" pattern silently ships a
half-migrated schema).
