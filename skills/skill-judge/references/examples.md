
# Dimension Examples

Good and bad examples for each evaluation dimension. Loaded for scoring calibration.

---

## D1: Knowledge Delta

**Red flags** (instant score <=5):
- "What is [basic concept]" sections
- Step-by-step tutorials for standard operations
- Explaining how to use common libraries
- Generic best practices ("write clean code", "handle errors")
- Definitions of industry-standard terms

**Green flags** (high knowledge delta):
- Decision trees for non-obvious choices ("when X fails, try Y because Z")
- Trade-offs only an expert would know ("A is faster but B handles edge case C")
- Edge cases from real-world experience
- "NEVER do X because [non-obvious reason]"
- Domain-specific thinking frameworks

---

## D2: Mindset + Appropriate Procedures

**Expert thinking patterns:**
```markdown
Before [action], ask yourself:
- **Purpose**: What problem does this solve? Who uses it?
- **Constraints**: What are the hidden requirements?
- **Differentiation**: What makes this solution memorable?
```

**Valuable domain procedures:**
```markdown
### Redlining Workflow (Claude wouldn't know this sequence)
1. Convert to markdown: `pandoc --track-changes=all`
2. Map text to XML: grep for text in document.xml
3. Implement changes in batches of 3-10
4. Pack and verify: check ALL changes were applied
```

**Redundant generic procedures:**
```markdown
Step 1: Open the file
Step 2: Find the section
Step 3: Make the change
Step 4: Save and test
```

---

## D3: Anti-Pattern Quality

**Expert anti-patterns** (specific + reason):
```markdown
NEVER use generic AI-generated aesthetics like:
- Overused font families (Inter, Roboto, Arial)
- Cliched color schemes (particularly purple gradients on white backgrounds)
- Predictable layouts and component patterns
- Default border-radius on everything
```

**Weak anti-patterns** (vague, no reasoning):
```markdown
Avoid making mistakes.
Be careful with edge cases.
Don't write bad code.
```

---

## D4: Specification Compliance

**Excellent description** (all three elements — WHAT, WHEN, KEYWORDS):
```yaml
description: "Comprehensive document creation, editing, and analysis with support
for tracked changes, comments, formatting preservation, and text extraction.
When Claude needs to work with professional documents (.docx files) for:
(1) Creating new documents, (2) Modifying or editing content,
(3) Working with tracked changes, (4) Adding comments, or any other document tasks"
```

**Poor description** (missing elements):
```yaml
description: "A helpful skill for various tasks"
```

---

## D5: Progressive Disclosure

**Good loading trigger** (embedded in workflow):
```markdown
### Creating New Document

**MANDATORY - READ ENTIRE FILE**: Before proceeding, you MUST read
[`docx-js.md`](docx-js.md) (~500 lines) completely from start to finish.
**NEVER set any range limits when reading this file.**

**Do NOT load** `ooxml.md` or `redlining.md` for this task.
```

**Bad loading trigger** (just listed):
```markdown
## References
- docx-js.md - for creating documents
- ooxml.md - for editing
- redlining.md - for tracking changes
```

---

## D6: Freedom Calibration

**High freedom** (creative tasks):
```markdown
Commit to a BOLD aesthetic direction. Pick an extreme: brutally minimal,
maximalist chaos, retro-futuristic, organic natural...
```

**Medium freedom** (judgment tasks):
```markdown
Review priority:
1. Security vulnerabilities (must fix)
2. Logic errors (must fix)
3. Performance issues (should fix)
4. Maintainability (optional)
```

**Low freedom** (fragile operations):
```markdown
**MANDATORY**: Use exact script in `scripts/create-doc.py`
Parameters: --title "X" --author "Y"
Do NOT modify the script.
```

---

## D7: Pattern Recognition

5 official patterns identified from 17+ official Skills:

| Pattern | ~Lines | Key Characteristics | Example |
|---------|--------|---------------------|---------|
| **Mindset** | ~50 | Thinking > technique, strong NEVER list, high freedom | frontend-design |
| **Navigation** | ~30 | Minimal SKILL.md, routes to sub-files | internal-comms |
| **Philosophy** | ~150 | Two-step: Philosophy then Express, emphasizes craft | canvas-design |
| **Process** | ~200 | Phased workflow, checkpoints, medium freedom | tdd-integration |
| **Tool** | ~300 | Decision trees, code examples, low freedom | docx, pdf, xlsx |

---

## D8: Practical Usability

**Good usability** (decision tree + fallback):
```markdown
| Task | Primary Tool | Fallback | When to Use Fallback |
|------|-------------|----------|----------------------|
| Read text | pdftotext | PyMuPDF | Need layout info |
| Extract tables | camelot-py | tabula-py | camelot fails |

**Common issues**:
- Scanned PDF: pdftotext returns blank -> Use OCR first
- Encrypted PDF: Permission error -> Use PyMuPDF with password
```

**Poor usability** (vague):
```markdown
Use appropriate tools for PDF processing.
Handle errors properly.
Consider edge cases.
```
