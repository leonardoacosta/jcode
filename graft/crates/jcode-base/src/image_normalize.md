# crates/jcode-base/src/image_normalize.rs

- PROVIDER_IMAGE_MEDIA_TYPES · constant · L17-L18 — pub const PROVIDER_IMAGE_MEDIA_TYPES: [&str; 4] =
- is_provider_media_type · function · L21-L25 — pub fn is_provider_media_type(media_type: &str) -> bool
- sniff_provider_media_type · function · L29-L43 — pub fn sniff_provider_media_type(bytes: &[u8]) -> Option<&'static str>
- sniff_other_format · function · L47-L78 — fn sniff_other_format(bytes: &[u8]) -> Option<&'static str>
- ProviderImage · struct · L82-L88 — pub struct ProviderImage
- normalize_image_bytes · function · L96-L120 — pub fn normalize_image_bytes(data: Vec<u8>) -> Result<ProviderImage, String>
- reencode_png · function · L122-L127 — fn reencode_png(data: &[u8]) -> Option<Vec<u8>>
- tests · module · L130-L216 — mod tests
- tiny_bmp · function · L133-L155 — fn tiny_bmp() -> Vec<u8>
- tiny_png · function · L157-L164 — fn tiny_png() -> Vec<u8>
- encode_as · function · L166-L173 — fn encode_as(format: image::ImageFormat) -> Vec<u8>
- provider_formats_pass_through_untouched · function · L176-L182 — fn provider_formats_pass_through_untouched()
- bmp_ico_and_tiff_are_converted_to_png · function · L185-L197 — fn bmp_ico_and_tiff_are_converted_to_png()
- undecodable_formats_are_refused_with_a_reason · function · L200-L215 — fn undecodable_formats_are_refused_with_a_reason()
