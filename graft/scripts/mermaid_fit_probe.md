# scripts/mermaid_fit_probe.py

- clamp · function · L22-L23 — def clamp(value: int, lo: int, hi: int) -> int
- utilization_percent · function · L26-L27 — def utilization_percent(used: int, total: int) -> float
- div_ceil · function · L30-L31 — def div_ceil(value: int, divisor: int) -> int
- axis_fill_zoom_percent · function · L34-L37 — def axis_fill_zoom_percent(available_cells: int, image_px: int, cell_px: int) -> int
- fit_zoom_percent_for_area · function · L40-L45 — def fit_zoom_percent_for_area(width_cells: int, height_cells: int, img_w_px: int, img_h_px: int, font_w: int, font_h: int) -> int
- vcenter_fitted · function · L48-L61 — def vcenter_fitted(width_cells: int, height_cells: int, img_w_px: int, img_h_px: int, font_w: int, font_h: int) -> dict[str, int]
- centered_viewport_scroll_cells · function · L64-L72 — def centered_viewport_scroll_cells(image_px: int, area_cells: int, zoom_percent: int, cell_px: int) -> int
- plan · function · L75-L125 — def plan(width_cells: int, height_cells: int, img_w_px: int, img_h_px: int, font_w: int, font_h: int) -> dict[str, Any]
- maybe_png_info · function · L128-L153 — def maybe_png_info(path: str | None) -> dict[str, Any] | None
- visual_fit_metrics · function · L156-L233 — def visual_fit_metrics( *, width_cells: int, height_cells: int, img_w_px: int, img_h_px: int, font_w: int, font_h: int, contain_rect: dict[str, int], alpha_bbox: list[int] | tuple[int, int, int, int] | None, ) -> dict[str, Any] | None
- main · function · L236-L291 — def main() -> int
- parse_pair · function · L258-L260 — def parse_pair(raw: str) -> tuple[int, int]
