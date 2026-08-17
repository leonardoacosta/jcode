# DI, HttpClient, config, and Azure auth

Dependency injection lifetimes, `IHttpClientFactory` typed clients, managed-identity wiring, and
`IOptions<T>` config binding. All registration lives in `Program.cs`.

## DI lifetimes

| Lifetime | Use for | Rule |
| --- | --- | --- |
| `Singleton` | `IMemoryCache`, `IOptions<T>`, stateless helpers, `TimeProvider` | One instance for the app lifetime. Must be thread-safe. |
| `Scoped` | Anything that depends on `DbContext`, per-request state | One instance per HTTP request. The default for services. |
| `Transient` | Cheap, stateless factories | New instance every resolution. |

**Captive-dependency trap:** never inject a `Scoped` service into a `Singleton` — the singleton
captures the first request's scoped instance and reuses it forever (stale `DbContext`, cross-request
data bleed). If a singleton needs scoped work, inject `IServiceScopeFactory` and create a scope
per unit of work.

```csharp
builder.Services.AddScoped<IDocumentsService, DocumentsService>();   // touches DbContext
builder.Services.AddSingleton<IClock, SystemClock>();                // stateless
builder.Services.AddTransient<IReportBuilder, ReportBuilder>();      // cheap factory
```

## IHttpClientFactory + typed clients

Never `new HttpClient()` per call — it leaks sockets (port exhaustion) and ignores DNS changes.
Use a typed client so the class gets an injected, factory-managed `HttpClient`.

```csharp
builder.Services.AddHttpClient<IDatabricksClient, DatabricksClient>(client =>
{
    client.BaseAddress = new Uri(builder.Configuration["Databricks:BaseUrl"]!);
    client.Timeout = TimeSpan.FromSeconds(120);
})
.AddStandardResilienceHandler();   // retry + circuit breaker + timeout (Microsoft.Extensions.Http.Resilience)
```

The consuming class receives the configured client by constructor injection:

```csharp
public sealed class DatabricksClient(HttpClient http) : IDatabricksClient
{
    public async Task<Result> QueryAsync(string sql, CancellationToken ct)
        => await http.PostAsJsonAsync("/sql", new { sql }, ct)
                     .ContinueWith(/* ... */);
}
```

`AddStandardResilienceHandler()` (from `Microsoft.Extensions.Http.Resilience`) is the .NET 8+
replacement for hand-rolled Polly policies — it gives retry with jitter, a circuit breaker, and a
per-attempt timeout out of the box.

## DefaultAzureCredential + managed identity

Prefer managed identity over any secret. `DefaultAzureCredential` walks a chain (env vars ->
workload identity -> managed identity -> Azure CLI) so the same code works in App Service and
locally.

```csharp
var credential = new DefaultAzureCredential();

// Acquire an access token for a resource (e.g. Databricks, ADLS, a downstream API)
var tokenRequest = new TokenRequestContext(
    new[] { "https://databricks.azure.com/.default" });
AccessToken token = await credential.GetTokenAsync(tokenRequest, ct);
```

Register a credential once and share it (it caches tokens):

```csharp
builder.Services.AddSingleton<TokenCredential>(new DefaultAzureCredential());
```

Never hardcode a PAT, client secret, or connection string. For the B&B specifics — which managed
identity resolves in which subscription, Key Vault RBAC, and the cloudpc SOCKS tunnel for local
runs — see the `bb-azure-ops` skill.

## IOptions<T> config binding

Bind config sections to typed options in `Program.cs`; inject `IOptions<T>` where consumed. Never
read `IConfiguration` directly inside a controller/service.

```csharp
// Options class
public sealed class DatabricksOptions
{
    public required string BaseUrl { get; init; }
    public int TimeoutSeconds { get; init; } = 120;
}

// Program.cs — bind + validate at startup
builder.Services
    .AddOptions<DatabricksOptions>()
    .Bind(builder.Configuration.GetSection("Databricks"))
    .ValidateDataAnnotations()
    .ValidateOnStart();   // fail fast at boot, not on first request

// Consume
public sealed class DatabricksClient(IOptions<DatabricksOptions> options) { ... }
```

Config precedence: `appsettings.json` -> `appsettings.{Environment}.json` -> env vars -> Key Vault
references. Secrets belong in Key Vault (referenced via
`@Microsoft.KeyVault(SecretUri=...)`), never in `appsettings.json`.
