# Custom HTTP JSON web search

For the built-in Kagi backend, see [Kagi web search](kagi-web-search.md).

Configure any HTTP endpoint returning JSON in `~/.jcode/config.toml`:

```toml
[websearch]
engine = "custom"
prefer_native = false
fallback_engines = []

[websearch.custom]
endpoint = "https://search.example.org/api"
method = "GET"
query_param = "q"
count_param = "count"
params = { format = "json" }
headers = { Accept = "application/json" }
header_env = { Authorization = "SEARCH_AUTH" }
results_pointer = "/data/results"
title_pointer = "/title"
url_pointer = "/url"
snippet_pointer = "/snippet"
```

`prefer_native` defaults to true. Set it to **false** to ensure supported model
providers do not replace the local custom engine with provider-native search.
Existing DuckDuckGo/Bing defaults are unchanged. Errors and empty results try
`fallback_engines` in order. The default fallback is Bing: set it to `[]` when
private queries must not be sent elsewhere. Custom can also be a fallback engine.

## Requests

`endpoint` must be an absolute HTTP(S) URL without userinfo or a fragment.
Prefer HTTPS for remote services. Only GET and POST are supported (upper or lower
case in configuration). GET is the default. `params` are static URL query
parameters for both methods and replace same-named endpoint query parameters.
For GET, the query and optional count are URL parameters. Dynamic values override
static values and same-named endpoint parameters. All parameters are URL encoded.

For POST, `body` is a static JSON object (default `{}`). The query is inserted as a
string at `query_param`; count is inserted as a JSON integer at `count_param`.
These are literal top-level keys, not pointers, and override existing body keys.
For example:

```toml
[websearch.custom]
endpoint = "https://search.example.org/api"
method = "POST"
query_param = "search"
count_param = "limit"
params = { version = "1" }
body = { options = { safe = true } }
```

`query_param` defaults to `q`. Omit `count_param` to send no count parameter.
When present it must be nonempty and different from `query_param`. Results are
still capped locally by `num_results` (tool default 8, maximum 20).

`headers` supplies static header values. Prefer `header_env` for credentials: it
maps header names to environment variable names and overrides static headers
case-insensitively. The environment variable contains the **complete** value,
including `Bearer ` if required. Missing/empty variables and invalid header
values fail before network access. Credentials are never copied into diagnostics.
Do not place credentials in URLs or bodies when header authentication is available.

Custom requests have a 30-second timeout and never follow redirects, including
same-host redirects, to prevent forwarding credentials to another destination.
Request errors strip reqwest URLs. HTTP errors show status only, not response
bodies, configured URLs, or headers. These safeguards do not replace trusting the
configured endpoint. Plain HTTP sends credentials unencrypted.

## JSON result mapping

Pointers follow JSON Pointer (RFC 6901): escape `/` as `~1` and `~` as `~0`.
`results_pointer` defaults to `/results` and must resolve to an array; use `""`
for a root array. Title, URL and snippet pointers are relative to each array item,
with defaults `/title`, `/url`, and `/snippet`. Nested paths such as `/link/href`
are supported. Missing/non-string/blank URLs are skipped. Missing/non-string or
blank titles fall back to the URL; missing/non-string snippets become empty.
Malformed JSON, invalid pointer syntax, or a missing/non-array results mapping
fails clearly and uses configured fallbacks. An empty valid array is no results.

## OmniRoute Firecrawl-first policy

For the OmniRoute gateway with the named policy installed, use this opt-in
recipe. Confirm deployment before activation: an older gateway may not implement
the selector. This documentation does not install or activate it.

```toml
[websearch]
engine = "custom"
prefer_native = false
fallback_engines = []

[websearch.custom]
endpoint = "https://pi5.tail296462.ts.net:8443/v1/search"
method = "POST"
query_param = "query"
count_param = "max_results"
body = { routing_policy = "firecrawl-first-brave-quota" }
header_env = { "x-api-key" = "JCODE_PROVIDER_OMNIPI_API_KEY" }
results_pointer = "/results"
title_pointer = "/title"
url_pointer = "/url"
snippet_pointer = "/snippet"
```

`JCODE_PROVIDER_OMNIPI_API_KEY` must be available to the process running the tool.
For `x-api-key`, its complete header value is the raw gateway key, without a
`Bearer ` prefix. Do not copy a credential into this file. Do not add `provider`
to `body`: the named policy owns provider selection. Omitting `routing_policy`
selects legacy gateway routing, not this policy.

The request body contains `routing_policy`, the query string as `query`, and the
requested result count as integer `max_results`. OmniRoute owns Firecrawl-first routing
and permits Brave only after classified Firecrawl billing or quota exhaustion.
Jcode does not classify gateway failures or implement that fallback itself.
HTTP errors and empty results have no local fallback with this recipe. Existing
custom-request limits still apply, including the 30-second timeout and no redirects.

## Provider-native search boundary

The custom endpoint handles Jcode's local `websearch` tool, not every provider's
hosted search. OpenAI Responses adds hosted `web_search` only for the first-party
API or ChatGPT backend, models without `codex` in their name, native-enabled
configuration, and a session that offers the local search tool. Anthropic adds its
server search tool only for OAuth or the first-party Messages API and a session
that offers search. Custom OpenAI/Anthropic gateway transports retain the local
search tool instead of automatically receiving hosted search definitions.

Set `prefer_native = false` with `engine = "custom"` to use the custom search
endpoint even when the model transport supports native search. Native search
requests intercepted inside a gateway follow that gateway's separately verified
routes and settings. This configuration does not enable or prove interception
for all model providers. Custom result mapping exposes only title, URL and
snippet, not gateway routing metadata outside those mapped fields.
