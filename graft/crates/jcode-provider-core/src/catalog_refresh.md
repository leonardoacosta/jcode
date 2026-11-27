# crates/jcode-provider-core/src/catalog_refresh.rs

- CatalogRouteKey · type · L5-L5 — type CatalogRouteKey = (String, String, String);
- CatalogRouteSnapshot · type · L6-L6 — type CatalogRouteSnapshot = (bool, String, Option<u64>);
- CatalogRouteMap · type · L7-L7 — type CatalogRouteMap = BTreeMap<CatalogRouteKey, CatalogRouteSnapshot>;
- ModelCatalogRefreshSummary · struct · L10-L24 — pub struct ModelCatalogRefreshSummary
- summarize_model_catalog_refresh · function · L26-L128 — pub fn summarize_model_catalog_refresh(
- is_display_only_age_suffix · function · L32-L38 — fn is_display_only_age_suffix(detail: &str) -> bool
- normalize_route_refresh_detail · function · L40-L54 — fn normalize_route_refresh_detail(detail: &str) -> String
