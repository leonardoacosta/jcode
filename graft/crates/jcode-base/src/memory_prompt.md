# crates/jcode-base/src/memory_prompt.rs

- MEMORY_CONTEXT_MAX_CHARS · constant · L1-L1 — const MEMORY_CONTEXT_MAX_CHARS: usize = 8_000;
- MEMORY_CONTEXT_MAX_MESSAGES · constant · L2-L2 — const MEMORY_CONTEXT_MAX_MESSAGES: usize = 12;
- MEMORY_CONTEXT_MAX_BLOCK_CHARS · constant · L3-L3 — const MEMORY_CONTEXT_MAX_BLOCK_CHARS: usize = 1_200;
- EXTRACTION_CONTEXT_MAX_MESSAGES · constant · L4-L4 — const EXTRACTION_CONTEXT_MAX_MESSAGES: usize = 40;
- EXTRACTION_CONTEXT_MAX_CHARS · constant · L5-L5 — const EXTRACTION_CONTEXT_MAX_CHARS: usize = 24_000;
- truncate_chars · function · L7-L12 — fn truncate_chars(value: &str, max_chars: usize) -> String
- format_content_block_for_relevance · function · L14-L51 — fn format_content_block_for_relevance(block: &crate::message::ContentBlock) -> Option<String>
- format_content_block_for_extraction · function · L53-L91 — fn format_content_block_for_extraction(block: &crate::message::ContentBlock) -> Option<String>
- format_message_context_with · function · L93-L118 — fn format_message_context_with(
- format_context_for_relevance · function · L121-L143 — pub fn format_context_for_relevance(messages: &[crate::message::Message]) -> String
- format_focused_query_for_relevance · function · L160-L163 — pub fn format_focused_query_for_relevance(messages: &[crate::message::Message]) -> String
- focus_query_text · function · L167-L233 — pub fn focus_query_text(raw: &str) -> String
- format_context_for_extraction · function · L238-L260 — pub(crate) fn format_context_for_extraction(messages: &[crate::message::Message]) -> String
