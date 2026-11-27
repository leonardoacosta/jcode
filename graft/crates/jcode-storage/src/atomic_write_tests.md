# crates/jcode-storage/src/atomic_write_tests.rs

- streamed_json_matches_encoding_and_preserves_backup · function · L5-L24 — fn streamed_json_matches_encoding_and_preserves_backup()
- FailingDocument · struct · L28-L28 — struct FailingDocument;
- serialize · function · L31-L40 — fn serialize<S: serde::Serializer>(
- failed_streamed_json_preserves_primary_backup_and_cleans_temp · function · L44-L64 — fn failed_streamed_json_preserves_primary_backup_and_cleans_temp()
- json_serialization_streams_before_document_is_complete · function · L67-L93 — fn json_serialization_streams_before_document_is_complete()
- StreamingDocument · struct · L68-L68 — struct StreamingDocument<'a>(&'a Path);
- serialize · function · L70-L85 — fn serialize<S: serde::Serializer>(
- failed_streamed_json_does_not_publish_new_file · function · L96-L102 — fn failed_streamed_json_does_not_publish_new_file()
- streamed_secret_json_keeps_primary_and_backup_owner_only · function · L106-L126 — fn streamed_secret_json_keeps_primary_and_backup_owner_only()
- raw_byte_writes_still_preserve_exact_bytes · function · L129-L135 — fn raw_byte_writes_still_preserve_exact_bytes()
