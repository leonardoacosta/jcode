# crates/jcode-base/src/mobile_sync/thumbnail_pipeline_tests.rs

- frozen_rgb · function · L7-L22 — fn frozen_rgb(image: DynamicImage) -> image::RgbImage
- frozen_thumbnail · function · L24-L62 — fn frozen_thumbnail(bytes: &[u8]) -> (Option<WireImage>, usize)
- fixture · function · L64-L86 — fn fixture(width: u32, height: u32, rgba: bool) -> DynamicImage
- full_thumbnail_pipeline_preserves_exact_base64_and_retry_output · function · L89-L111 — fn full_thumbnail_pipeline_preserves_exact_base64_and_retry_output()
- full_thumbnail_pipeline_preserves_invalid_image_rejection · function · L114-L121 — fn full_thumbnail_pipeline_preserves_invalid_image_rejection()
- full_thumbnail_pipeline_abba_benchmark · function · L125-L180 — fn full_thumbnail_pipeline_abba_benchmark()
