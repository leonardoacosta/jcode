# crates/jcode-message-types/src/tool_prompt_size_tests.rs

- Counter · struct · L6-L6 — struct Counter(usize);
- write · function · L8-L11 — fn write(&mut self, bytes: &[u8]) -> io::Result<usize>
- write_all · function · L12-L15 — fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>
- flush · function · L16-L18 — fn flush(&mut self) -> io::Result<()>
- writer · function · L20-L24 — fn writer(value: &Value) -> usize
- recursive · function · L25-L43 — fn recursive(value: &Value) -> usize
- cases · function · L44-L73 — fn cases() -> Vec<(&'static str, Value)>
- candidates_match_frozen_serialization · function · L76-L82 — fn candidates_match_frozen_serialization()
- exact_bytes_after_schema_and_tool_mutation · function · L85-L127 — fn exact_bytes_after_schema_and_tool_mutation()
- frozen · function · L86-L91 — fn frozen(tool: &ToolDefinition) -> usize
- generated_string_and_numeric_byte_parity · function · L130-L145 — fn generated_string_and_numeric_byte_parity()
- paired_schema_count_benchmark · function · L149-L186 — fn paired_schema_count_benchmark()
- frozen · function · L150-L152 — fn frozen(value: &Value) -> usize
- measure · function · L153-L159 — fn measure(f: fn(&Value) -> usize, value: &Value, iterations: usize) -> f64
