# crates/jcode-command-risk/src/gate.rs

- GateOutcome · enum · L25-L32 — pub enum GateOutcome
- Justification · struct · L36-L39 — pub struct Justification
- is_substantive · function · L47-L71 — pub fn is_substantive(&self) -> bool
- EMPTY_AFFIRMATIONS · constant · L55-L67 — const EMPTY_AFFIRMATIONS: &[&str] = &[
- MIN_JUSTIFICATION_LEN · constant · L75-L75 — const MIN_JUSTIFICATION_LEN: usize = 25;
- gate · function · L78-L99 — pub fn gate(assessment: &RiskAssessment, justification: &Justification) -> GateOutcome
- reflection_prompt · function · L107-L122 — fn reflection_prompt(assessment: &RiskAssessment) -> String
- gate_tests · module · L126-L126 — mod gate_tests;
