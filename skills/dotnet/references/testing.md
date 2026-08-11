# Testing

xUnit structure, `WebApplicationFactory` integration tests, real-DB via Testcontainers, mocking
boundaries. The E2E phase gate is `dotnet test`.

## xUnit structure

xUnit is the fleet default. One test class per unit under test; `[Fact]` for a single case,
`[Theory]` + `[InlineData]` for parameterized cases. Arrange-Act-Assert, one behavior per test.

```csharp
public sealed class DocumentValidatorTests
{
    [Fact]
    public void Rejects_empty_title()
    {
        var result = DocumentValidator.Validate(new() { Title = "" });

        Assert.False(result.IsValid);
        Assert.Contains("Title", result.Errors.Single().Field);
    }

    [Theory]
    [InlineData(0)]
    [InlineData(101)]
    public void Rejects_priority_out_of_range(int priority)
    {
        var result = DocumentValidator.Validate(new() { Title = "ok", Priority = priority });
        Assert.False(result.IsValid);
    }
}
```

Assertion libraries: `Assert.*` is fine; `FluentAssertions` or `Shouldly` are common if the repo
already uses one — match existing style, do not introduce a second.

## WebApplicationFactory integration tests

`WebApplicationFactory<TEntryPoint>` boots the real app in-memory and hands you an `HttpClient`
that exercises the full middleware pipeline — routing, model binding, filters, auth.

```csharp
public sealed class DocumentsApiTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public DocumentsApiTests(WebApplicationFactory<Program> factory)
        => _client = factory.CreateClient();

    [Fact]
    public async Task Get_unknown_document_returns_404()
    {
        var response = await _client.GetAsync($"/documents/{Guid.NewGuid()}");
        Assert.Equal(HttpStatusCode.NotFound, response.StatusCode);
    }
}
```

Override services (swap the real credential, stub a downstream client) via
`factory.WithWebHostBuilder(b => b.ConfigureTestServices(...))`. Requires `Program` to be
reachable — add `public partial class Program;` at the bottom of `Program.cs` for the top-level
statements form.

## Real DB via Testcontainers

Prefer a real database over an in-memory provider — the EF in-memory provider does not enforce
relational constraints (unique keys, FKs, transactions) and hides real bugs. Testcontainers spins
up a disposable SQL Server / Postgres container per test class.

```csharp
public sealed class DatabaseFixture : IAsyncLifetime
{
    private readonly MsSqlContainer _db = new MsSqlBuilder().Build();
    public string ConnectionString => _db.GetConnectionString();

    public async Task InitializeAsync()
    {
        await _db.StartAsync();
        // apply migrations against the fresh container
    }

    public Task DisposeAsync() => _db.DisposeAsync().AsTask();
}
```

Point the `WebApplicationFactory` at the container's connection string via `ConfigureTestServices`
so integration tests hit real relational behavior.

## Mocking boundaries

| Mock | Do NOT mock |
| --- | --- |
| External HTTP clients (downstream APIs) | Your own `DbContext` — use a real DB (Testcontainers) |
| Non-deterministic sources (`TimeProvider`/`IClock`, GUID/random) | EF Core query behavior |
| Cloud SDKs you cannot reach in test (Key Vault, ADLS) | The framework (routing, model binding) |

Design for testability: depend on interfaces (`IFoo`) not concretes, inject `TimeProvider` for
testable time, and propagate `CancellationToken` through every async layer so cancellation is
observable in tests.

`WireMock.Net` or a stub `HttpMessageHandler` mocks a downstream HTTP dependency without mocking
your own service surface.
