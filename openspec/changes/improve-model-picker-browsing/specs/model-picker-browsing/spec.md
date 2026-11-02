## ADDED Requirements

### Requirement: Explicit composable browsing controls
The `/model` picker SHALL expose All/Favorites scope, All providers/a selected provider, and Alphabetical/Most used sorting. Provider and favorite filters SHALL compose with text search and SHALL operate on the base catalog returned by the existing configured catalog loader. Existing zero-match allowlist fallback and current-route exceptions SHALL remain explicit inherited catalog behavior, not permission to silently widen an interactive provider filter.

#### Scenario: Combined filters
- **WHEN** the user selects Favorites, a provider and a search query
- **THEN** only favorites matching that provider and query within the configured catalog SHALL appear

#### Scenario: No matching models
- **WHEN** the selected filters match no model
- **THEN** the picker SHALL show an empty state and reset controls without silently widening the filters

#### Scenario: Existing zero-match configuration fallback
- **WHEN** a configured provider allowlist matches no route and the existing loader returns its fallback catalog
- **THEN** the picker SHALL label the configuration fallback and subsequent interactive provider selection SHALL still restrict results to that provider

#### Scenario: Current favorite removed
- **WHEN** the user unfavorites the focused or sole row while Favorites scope is active
- **THEN** the visible list SHALL immediately refilter, focus SHALL move to a valid remaining identity or the empty state, and Enter SHALL NOT select the removed row through a stale index

### Requirement: Stable and explainable sorting
Alphabetical sorting SHALL use case-insensitive displayed names with stable identity tie-breaks. Most used SHALL use the existing prospective count of agent turns with at least one persisted assistant response on the model route, aggregated across reasoning efforts, and identify any fallback to picker-selection counts. It SHALL NOT label that count as raw requests or all-time usage. Search SHALL narrow results without replacing the selected sort.

#### Scenario: Stable equal counts
- **WHEN** two matching entries have equal tracked-turn counts but different last-used timestamps or legacy selection counts
- **THEN** repeated refreshes SHALL preserve deterministic ordering through stable name and identity tie-breaks

#### Scenario: Missing usage evidence
- **WHEN** tracked-turn evidence is unavailable
- **THEN** the picker SHALL identify selection-count fallback or no usage data and SHALL NOT imply successful model calls or spend

### Requirement: Preserve selection and favorites semantics
Browsing SHALL NOT change the active model, account, reasoning effort, default model or favorite state without the corresponding explicit action. Existing favorite identity and persistence semantics SHALL be reused and documented.

#### Scenario: Current model excluded
- **WHEN** a filter excludes the active model
- **THEN** the active model SHALL remain identified separately and SHALL NOT appear as a matching result

#### Scenario: Favorite route unavailable
- **WHEN** a favorite has no usable route or disappeared from the catalog
- **THEN** its unavailability SHALL be explained without silently selecting a different account or provider

### Requirement: Honest route-scoped health evidence
The picker SHALL distinguish route availability from observed health. Health evidence SHALL identify its route/model/account scope, source and observation age; missing or expired evidence SHALL be unknown or stale, never healthy by inference.

#### Scenario: Configured but untested route
- **WHEN** credentials are configured but no recent request outcome exists for the model route
- **THEN** availability MAY be shown but health SHALL remain unknown

#### Scenario: One route fails
- **WHEN** one route returns a model or authorization failure
- **THEN** the failure SHALL NOT mark another route/account for the same model unhealthy without corresponding evidence

### Requirement: Provider quota retains account and window attribution
Quota SHALL be displayed only for the attributable serving provider/account, with measurement age, usage window and reset information when available. Bars SHALL reuse the info widget renderer and used-percentage convention.

#### Scenario: Supported quota collector
- **WHEN** a matching cached provider/account usage result contains a limit
- **THEN** the picker SHALL show that limit and its age/window without presenting it as an exact per-model token balance

#### Scenario: Unsupported or ambiguous quota
- **WHEN** quota is unsupported, stale or cannot be tied to the selected route's account
- **THEN** the picker SHALL show unknown/stale with a reason and SHALL NOT substitute another account's quota

### Requirement: Nonblocking and nonbillable browsing
Opening, filtering and sorting SHALL use existing cached catalog/usage state and SHALL NOT trigger model inference or a per-model probe fan-out. Explicit supported telemetry refresh SHALL run outside the input/render path and retain the user's filters and stable selection identity.

#### Scenario: Slow telemetry refresh
- **WHEN** an explicit refresh is delayed or fails
- **THEN** typing and navigation SHALL remain responsive and cached data SHALL retain its age or error indicator

#### Scenario: Catalog updates during selection
- **WHEN** a catalog response arrives after filters or selection changed
- **THEN** it SHALL NOT apply a stale row index as a model selection or reset current browsing controls

### Requirement: Remote and narrow-terminal parity
Remote telemetry SHALL originate from the serving host and credential owner. Chrome SHALL fit available Unicode display cells without wrapping; labels SHALL remain understandable without color.

#### Scenario: Remote status unsupported
- **WHEN** an older remote server cannot supply account-scoped status or persistence
- **THEN** the client SHALL show the capability as unavailable and SHALL NOT read client-local credentials or write remote state into local preference files

#### Scenario: Narrow or monochrome terminal
- **WHEN** the picker renders at 60 columns with wide Unicode names or with color disabled
- **THEN** controls and status SHALL remain single-row or intentionally stacked, truncate by display cells, and retain textual status labels
