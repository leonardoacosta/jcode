# crates/jcode-core/src/sse_stream_integrity_tests.rs

- multibyte_char_split_across_chunks_is_not_dropped · function · L5-L18 — fn multibyte_char_split_across_chunks_is_not_dropped()
- every_single_byte_split_point_round_trips · function · L21-L32 — fn every_single_byte_split_point_round_trips()
- byte_at_a_time_delivery_round_trips · function · L35-L44 — fn byte_at_a_time_delivery_round_trips()
- genuinely_invalid_bytes_do_not_discard_surrounding_text · function · L47-L56 — fn genuinely_invalid_bytes_do_not_discard_surrounding_text()
- well_formed_single_object_is_not_treated_as_concatenated · function · L59-L61 — fn well_formed_single_object_is_not_treated_as_concatenated()
- concatenated_objects_are_split · function · L64-L68 — fn concatenated_objects_are_split()
- concatenation_with_embedded_data_prefix_is_split · function · L71-L74 — fn concatenation_with_embedded_data_prefix_is_split()
- braces_inside_strings_do_not_confuse_the_splitter · function · L77-L84 — fn braces_inside_strings_do_not_confuse_the_splitter()
- escaped_quote_inside_string_is_handled · function · L87-L94 — fn escaped_quote_inside_string_is_handled()
- truncated_trailing_object_is_reported_for_carry_over · function · L97-L105 — fn truncated_trailing_object_is_reported_for_carry_over()
- split_object_rejoins_into_valid_json · function · L108-L121 — fn split_object_rejoins_into_valid_json()
