# crates/jcode-base/src/mobile_sync/thumbnail_identity_tests.rs

- reference · function · L4-L6 — fn reference(image: DynamicImage) -> RgbImage
- jpeg · function · L8-L14 — fn jpeg(image: &RgbImage) -> Vec<u8>
- compare · function · L16-L25 — fn compare(image: DynamicImage, check_jpeg: bool)
- pattern · function · L27-L50 — fn pattern(width: u32, height: u32, rgba: bool, seed: u32) -> DynamicImage
- thumbnail_identity_exhaustive_channel_values · function · L53-L72 — fn thumbnail_identity_exhaustive_channel_values()
- thumbnail_identity_boundary_patterns_and_random_dimensions · function · L75-L95 — fn thumbnail_identity_boundary_patterns_and_random_dimensions()
- thumbnail_identity_jpeg_identity_and_fallback · function · L98-L111 — fn thumbnail_identity_jpeg_identity_and_fallback()
- thumbnail_identity_trailing_storage_matches_reference · function · L114-L121 — fn thumbnail_identity_trailing_storage_matches_reference()
- thumbnail_identity_does_not_skip_upscaling · function · L124-L128 — fn thumbnail_identity_does_not_skip_upscaling()
- thumbnail_identity_other_formats_and_empty_images_use_reference · function · L131-L151 — fn thumbnail_identity_other_formats_and_empty_images_use_reference()
- thumbnail_identity_paired_benchmark · function · L155-L202 — fn thumbnail_identity_paired_benchmark()
