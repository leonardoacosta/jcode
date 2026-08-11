# Controllers and APIs

Controller vs minimal API, model binding + validation, `ProblemDetails` errors, versioning.

## Controller vs minimal API

| Choose | When |
| --- | --- |
| **Controllers** (`[ApiController]`) | Rich endpoints, filters, model-binding attributes, existing controller-based repo. The B&B fleet default (dc, sc, etc.). |
| **Minimal APIs** (`app.MapGet`) | Small services, few endpoints, low ceremony. Fine for a lean worker's health/admin surface. |

Match the existing repo — do not mix styles for the same resource. Keep controllers thin; put
logic in injected services so it stays testable.

### Controller shape

```csharp
[ApiController]
[Route("documents")]
[Authorize]
public sealed class DocumentsController(
    IDocumentsService service,
    ILogger<DocumentsController> logger) : ControllerBase
{
    [HttpGet("{id:guid}")]
    [ProducesResponseType(typeof(DocumentDto), StatusCodes.Status200OK)]
    [ProducesResponseType(StatusCodes.Status404NotFound)]
    public async Task<IActionResult> Get(Guid id, CancellationToken ct)
    {
        var doc = await service.GetAsync(id, ct);
        return doc is null ? NotFound() : Ok(doc);
    }
}
```

### Minimal API shape

```csharp
app.MapGet("/documents/{id:guid}",
    async (Guid id, IDocumentsService service, CancellationToken ct) =>
    {
        var doc = await service.GetAsync(id, ct);
        return doc is null ? Results.NotFound() : Results.Ok(doc);
    })
   .WithName("GetDocument")
   .Produces<DocumentDto>()
   .Produces(StatusCodes.Status404NotFound);
```

## Model binding + validation

`[ApiController]` auto-triggers model validation and returns a `ValidationProblemDetails` (400) on
failure — no manual `if (!ModelState.IsValid)` needed. Annotate the request model:

```csharp
public sealed record CreateDocumentRequest
{
    [Required, StringLength(200)]
    public required string Title { get; init; }

    [Range(1, 100)]
    public int Priority { get; init; }
}
```

Bind sources explicitly when the default inference is ambiguous: `[FromQuery]`, `[FromRoute]`,
`[FromBody]`, `[FromServices]`. Clamp pagination and whitelist sort columns server-side — reject
unknown values with a 400 `ProblemDetails`, never interpolate them into a query.

## ProblemDetails error handling

Return machine-readable RFC 7807 problem responses, never raw strings. Wire the global handler once:

```csharp
builder.Services.AddProblemDetails();
app.UseExceptionHandler();   // unhandled exceptions -> ProblemDetails
app.UseStatusCodePages();
```

For a deliberate error response from an action:

```csharp
return Problem(
    title: "Invalid sortBy",
    detail: $"sortBy must be one of: {string.Join(", ", AllowedSortColumns)}",
    statusCode: StatusCodes.Status400BadRequest);
```

Let unhandled exceptions reach the middleware — do not `try { } catch (Exception) { }` and swallow.
Catch only what you can meaningfully handle and recover from.

## Versioning

Use `Asp.Versioning.Mvc` (the successor to `Microsoft.AspNetCore.Mvc.Versioning`):

```csharp
builder.Services.AddApiVersioning(options =>
{
    options.DefaultApiVersion = new ApiVersion(1, 0);
    options.AssumeDefaultVersionWhenUnspecified = true;
    options.ReportApiVersions = true;
})
.AddMvc();

[ApiVersion("1.0")]
[Route("v{version:apiVersion}/documents")]
public sealed class DocumentsV1Controller : ControllerBase { ... }
```

Prefer URL-segment versioning (`/v1/...`) for public APIs — it is cache-friendly and unambiguous.
