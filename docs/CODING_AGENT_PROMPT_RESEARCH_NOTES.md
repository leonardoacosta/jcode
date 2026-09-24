# Earlier prompt-source research and provenance notes

Recorded: 2026-09-24.

These two public-source reports preserve the earlier broad investigation before
it narrowed to evaluated sections of Jcode's existing prompts. They are historical
research artifacts, not freshly reverified source claims. Later findings and
recommendations in [the main report](CODING_AGENT_PROMPT_RESEARCH.md) take precedence.
In particular, the later report includes the revised AGENTS.md category ablations,
uses “prompt variant” for mobile comparisons that change several things together,
and distinguishes the checked-in prompt from effective session overrides.

## Additional leads and access limitations

- Pi Harness's initial 55-case / five-runs / seven-category / 88.7%-at-baseline
  scorecard claim below was not independently rechecked during documentation.
  The one-case `coding-055` score change from 0.2 to 0.8 is a project-local
  scaffold report, not a controlled general comparison.
- AgentCN was interpreted as `shadcn-labs/agentcn`. Earlier discovery used
  agentcn.dev, while later retrieval used agentcn.run. These domains are preserved
  as historical citations, not asserted to be interchangeable current endpoints.
- https://jcode.sh/bench was explicitly unfinished. Neither it nor the swarm page
  establishes a prompt-section comparison.
- https://github.com/ying1973/SwarmBench remained an uninspected lead.
- Agentless and multi-agent failure studies remained leads without a completed
  source-backed section-level synthesis. No claims from them drive recommendations.
- GEPA HTML retrieval at https://arxiv.org/html/2507.19457v1 exceeded the fetch
  size limit. It supplies no verified numeric result in this report.
- Firecrawl/Aperture routes returned 404 and DuckDuckGo encountered anti-bot
  challenges. Known URLs were retrieved with available fetch tools.
- Large successful fetches were often truncated. Fetch success alone was not
  accepted as verification. Later targeted extraction recovered mobile Appendix C
  and the revised AGENTS.md Appendix B / Table 7.
- Research-worker reports were not reliably retrievable through summary/context
  tools. An older worker reported inability to verify sources. Worker availability
  or readiness was not treated as supporting evidence.
- No evaluations were executed. Prompt files and runtime behavior were not changed.

## Historical report 1

# Research brief: agent system and swarm prompts

**Scope:** Publicly documented prompt artifacts and prior evaluation work relevant to Pi, Jcode, and AgentCN. I did not run evaluations. The web search integration returned an endpoint error, so I used the available web-search tool and fetched primary project pages where possible. This is a public-web scan, not proof that no private or unindexed evaluation exists.

## Findings matrix

| Project or work | What is publicly available | Evaluation evidence found | Relevance to replacing Jcode prompts |
|---|---|---|---|
| **Pi coding agent** | Pi documents a minimal system prompt, project-level `SYSTEM.md` replacement or append, `AGENTS.md`, prompt templates, and extension-based context injection. Its site links directly to the prompt source. Pi’s core deliberately omits built-in sub-agents, so there is no single canonical Pi swarm prompt to compare. | Pi’s public docs describe customization, but the Pi materials I inspected did not themselves provide a controlled comparison of alternative system prompts. | Strong source for alternative **single-agent prompt structure** and customization patterns. Less directly applicable to swarm instructions because Pi leaves orchestration to extensions or external tooling. [Pi docs](https://pi.dev/) |
| **Pi Harness (`forrestbthomas/pi-harness`)** | A public harness around Pi, with a versioned eval contract, task cases, scorecards, and a measure → diagnose → improve → re-measure workflow. | **Yes.** Its README reports a dated scorecard from 2026-08-15: 55 live cases, 5 runs per case, 7 categories, 88.7% at/above baseline, with model, judge, and dataset provenance. It explicitly calls the suite a one-person demo, not a general benchmark, and says its scores are evidence about its mechanism rather than agent quality. | Most directly relevant precedent for evaluating prompt changes in a Pi-based agent. It is not evidence that its prompts outperform Jcode’s, nor a head-to-head Jcode/Pi study. [Repository](https://github.com/forrestbthomas/pi-harness) |
| **Jcode** | The public README describes swarm coordination and points to swarm architecture documentation. The project also has a public benchmark page, but the retrieved material here does not establish that it compares **system-prompt or swarm-prompt variants**. | I found no public evidence in the inspected pages of a controlled prompt A/B evaluation between Pi and Jcode. That is a limit of this scan, not a claim that none exists. | Jcode’s current system and swarm prompt files are the natural baselines for the requested future comparison. Publicly described harness features make clear that a swarm prompt would need to be evaluated in the context of the coordination tools, not as prose alone. [Repository](https://github.com/1jehuang/jcode) · [Swarm overview](https://jcode.sh/swarm) |
| **AgentCN** | AgentCN describes itself as a registry and CLI for installing agent recipes into Next.js projects. Its docs describe source, instructions, tools, and wiring as part of those recipes. | **No AgentCN-specific prompt evaluation surfaced in the public results I inspected.** AgentCN’s evaluated status remains unverified. Its public positioning is primarily an agent-recipe distribution kit, not a coding-agent harness evaluation project. | Worth inspecting for **reusable agent instruction and workflow patterns**, but do not treat it as an evaluated replacement prompt unless a specific benchmark or evaluation is found. [Docs](https://www.agentcn.dev/docs) · [GitHub](https://github.com/shadcn-labs/agentcn) |
| **General agent-evaluation guidance** | Anthropic’s engineering article discusses evaluation design for agents, including matching measurements to agent complexity. | General methodology guidance, not an evaluation of Pi, Jcode, or AgentCN prompts. | Useful background if the team later designs its own prompt comparison. [Anthropic: Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) |

## What the research supports

- **There is a concrete public precedent for evaluating a Pi-based coding setup:** Pi Harness reports repeatable, versioned cases and scorecards, while carefully limiting what its results claim.
- **Pi offers prompt replacement and extension mechanisms**, but its core philosophy is to keep the harness small. Its public docs do not provide a standard swarm prompt.
- **AgentCN is a source-distribution approach for product agents.** The public material inspected does not establish that its prompts have been benchmarked, and its use case differs from coding-agent orchestration.
- **A prompt comparison should separate system prompt from swarm prompt.** Pi’s documented baseline is principally a single-agent prompt. Jcode’s swarm prompt operates alongside orchestration, shared-repo awareness, messaging, and task coordination.

## Caveats and open questions

| Question | Current status |
|---|---|
| Has AgentCN ever run a public prompt evaluation? | No result found in this scan. Not established absent. |
| Are there public Pi-vs-Jcode system-prompt comparisons? | None found in inspected sources. |
| Does Jcode’s public benchmark measure prompt variants? | Not confirmed from the material retrieved. |
| Which specific prompt files should be treated as the current Jcode baselines? | Not inspected in this public-web pass; the active local/global prompt files may differ by installation and project. |
| Are public Pi Harness results transferable to Jcode? | No. They evaluate a particular Pi configuration and its own cases, not Jcode’s runtime or swarm behavior. |

## Sources

- [Pi Coding Agent](https://pi.dev/) — official description of its minimal prompt, `SYSTEM.md`, `AGENTS.md`, templates, and extension model.
- [Pi Harness](https://github.com/forrestbthomas/pi-harness) — public evaluation harness and its dated scorecard, methodology, and stated limits.
- [Jcode repository](https://github.com/1jehuang/jcode) — public project overview and swarm description.
- [Jcode Swarm](https://jcode.sh/swarm) — public description of coordination; page labels itself unfinished.
- [AgentCN docs](https://www.agentcn.dev/docs) — official description of its registry, CLI, agent recipes, and demo.
- [AgentCN repository](https://github.com/shadcn-labs/agentcn) — public source repository.
- [Anthropic agent evals](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) — general evaluation guidance.

**Bottom line:** Pi Harness is the clearest discovered example of public, repeatable evaluation in the Pi ecosystem. The scan did not find public evidence that AgentCN prompts have been evaluated or that Jcode and Pi prompts have been compared head-to-head.

---

## Historical report 2

# Prompt alternatives and evaluations for Pi, Jcode, and AgentCN

## Executive summary

**There is direct public evidence that prompt wording can be tested rather than debated.** Two useful precedents are assistant-ui’s small A/B harness for individual guidance lines and the SWE-Bench Mobile paper’s 12-variant prompt ablation. Neither evaluates Pi against Jcode, and neither establishes which prompt is best for either of those agents.

**Pi and Jcode expose different prompt-customization layers.** Pi documents replacing or appending to its system prompt, along with project instructions, skills, prompt templates, packages, and extensions. Jcode’s public docs describe a built-in system prompt with user-editable overrides and overlays, plus a separately configurable swarm prompt. **AgentCN is a collection of agent recipes, not a coding-agent prompt benchmark** in the sources reviewed. I found no AgentCN-specific prompt evaluation, but that is a search result, not proof that none exists.

No evaluations were run for this research.

## 1. Prompt and instruction sources

| System | Publicly documented prompt or instruction sources | What they can vary | Evidence and limits |
|---|---|---|---|
| **Pi** | User or project `SYSTEM.md` replaces the default system prompt; `APPEND_SYSTEM.md` adds instructions. Pi also loads `AGENTS.md`/`CLAUDE.md` context files. Skills load specialized instructions on demand; prompt templates are reusable user messages; extensions can add instructions or transform context. | Core system instructions, appended guidance, project context, task-specific skills and templates, and executable extension behavior. | [Configuration](https://pi.dev/docs/latest/configuration), [Skills](https://pi.dev/docs/latest/skills), [Prompt Templates](https://pi.dev/docs/latest/prompt-templates), [Extensions](https://pi.dev/docs/latest/extensions), [How Pi Works](https://pi.dev/docs/latest/how-pi-works). These are official customization docs, not comparative prompt results. |
| **Jcode** | `docs/SYSTEM_PROMPT_CONFIG.md` describes prompt layers and override behavior. The built-in prompt is `crates/jcode-base/src/prompt/system_prompt.md`; the swarm routing prompt is `crates/jcode-base/src/prompt/swarm_prompt.md`. The configuration docs identify project and global overrides. | Base system behavior and swarm model-routing guidance, with separate prompt surfaces for user customization. | [System-prompt configuration](https://github.com/1jehuang/jcode/blob/main/docs/SYSTEM_PROMPT_CONFIG.md), [built-in system prompt](https://github.com/1jehuang/jcode/blob/main/crates/jcode-base/src/prompt/system_prompt.md), [swarm prompt](https://github.com/1jehuang/jcode/blob/main/crates/jcode-base/src/prompt/swarm_prompt.md). Repository source was inspected locally; web retrieval of the Jcode GitHub paths failed in this run, so those links are repository-path citations rather than independently fetched page contents. |
| **AgentCN** | The public project describes installable agent recipes containing instructions, tools, skills, and workflows, copied into the user’s codebase. It lists Eve, Flue, Mastra, and LangGraph recipes. | Recipe-level instructions and agent implementation components, which users can customize after copying. | [AgentCN repository](https://github.com/shadcn-labs/agentcn), [AgentCN docs](https://www.agentcn.run/docs.md). The repository advertises live previews, but that is not an evaluation of prompt variants. |
| **Pi Harness** | A separate Pi-based harness exposes task briefs, executor prompts, validation and review scripts, and runtime enforcement. | Task-specific prompt and harness configuration. | [pi-harness](https://github.com/forrestbthomas/pi-harness). Its reported score changes are project-specific and should not be read as Pi-versus-Jcode results. |

**Practical distinction:** A system prompt replacement is not equivalent to adding an `AGENTS.md` rule, loading a skill, or changing tool/runtime behavior. Evaluation results are most informative when they state which layer changed.

## 2. Public evidence of prompt evaluation

| Source | What was compared | Reported result | Relevance and caveats |
|---|---|---|---|
| [assistant-ui prompt evals](https://github.com/assistant-ui/assistant-ui/blob/main/evals/README.md) | A seeded code-edit task with baseline and alternative guidance phrasings, judged against a rubric. The README says the agent runs in an isolated sandbox with only the injected system guidance, and a fresh Claude instance judges results. | For removing stale history comments, the selected concise instruction reached about **94%** on Opus in the reported confirmation runs, versus a **0%** baseline; performance differed by model and wording. Other phrasings did not produce the same result. | Strong precedent for testing one instruction against a baseline. It covers a narrow behavior and small trial counts, not broad coding quality or Pi/Jcode prompts. |
| [SWE-Bench Mobile paper](https://arxiv.org/html/2602.09540) | Twelve prompt strategies, tested with **Claude Code + GLM 4.6** on a 50-task iOS feature benchmark. | “Defensive Programming” had **10% task success** and **26.7% test pass rate**, versus **10%** and **19.3%** for baseline. Several more elaborate prompts scored **4% task success**. The paper reports a 7.4 percentage-point test-pass increase, not an increase in fully solved tasks. | Direct prompt ablation, but only for one agent-model pairing. Evaluation uses task-specific tests on patch diffs rather than compiling or running the app. The authors explicitly note limited prompt/model coverage. |
| [Pi Harness](https://github.com/forrestbthomas/pi-harness) | A Pi-based harness with task scorecards and CI drift checks. The public repository describes a prompt-scaffold fix for one debugging case, `coding-055`. | The repository reports that the case moved from **0.2 to 0.8** after a prompt-scaffold fix. | A useful project-local example of iterating on prompts with a scorecard. The score is specific to that harness and case, not a general agent benchmark or controlled Pi-vs-Jcode comparison. |
| [Jcode discovery benchmarks](https://github.com/1jehuang/jcode/blob/main/docs/DISCOVERY_BENCHMARK.md) and [call-rate benchmark](https://github.com/1jehuang/jcode/blob/main/docs/DISCOVERY_RATE_BENCHMARK.md) | Repository docs describe measuring whether Jcode discovers expected integrations and whether it calls the integration tool when appropriate. | These measure specific discovery policies, not general system-prompt quality. | The local docs were inspected. They show Jcode has evaluation infrastructure for bounded behaviors, but do not establish that candidate system or swarm prompts have been compared. |
| [Jcode swarm benchmark script](https://github.com/1jehuang/jcode/blob/main/scripts/benchmark_swarm.py) | The script describes comparing single-agent and swarm performance on an Anthropic performance take-home task. | The inspected script is a benchmark precedent for swarm-vs-single execution. | It is not, by itself, a controlled comparison of alternative swarm prompts. |
| [Jcode Terminal-Bench notes](https://github.com/1jehuang/jcode/blob/main/docs/TERMINAL_BENCH.md) | Documentation describes running Jcode through Harbor on Terminal-Bench 2.0. | No prompt-variant conclusion was established from the material inspected. | Benchmarking an agent is not the same as evaluating prompt alternatives. |

## 3. Prompt and swarm prompt alternatives

| Alternative | What it changes | Example source | Evaluation evidence found |
|---|---|---|---|
| **Replace the system prompt** | The base instructions for a run or project. | Pi `SYSTEM.md`; Jcode system-prompt override. | Pi documents the mechanism; no Pi prompt-variant result found in the reviewed sources. |
| **Append focused guidance** | Adds a rule without replacing the base prompt. | Pi `APPEND_SYSTEM.md`; assistant-ui injected guidance candidates. | assistant-ui provides a controlled, narrow A/B precedent. |
| **Project instruction files** | Adds repository-specific conventions and constraints. | Pi context files; Jcode’s `AGENTS.md` layer. | No direct Pi-versus-Jcode comparison found. |
| **On-demand skill or recipe** | Loads detailed task guidance only when relevant. | Pi Skills; AgentCN recipes. | Documentation and examples establish availability, not comparative effectiveness. |
| **Reusable prompt template** | Supplies a repeatable task message, such as a review request. | Pi Prompt Templates. | Useful for standardizing task inputs; not a system-prompt ablation by itself. |
| **Swarm routing prompt** | Changes how workers are selected or assigned. | Jcode’s `swarm_prompt.md`; Pi extensions/packages can implement executable coordination patterns. | Jcode’s swarm-vs-single benchmark is related to architecture, but the inspected evidence does not isolate the swarm prompt as the causal variable. |
| **Harness/tool-surface changes** | Changes capabilities, tool interaction, context management, or stopping behavior rather than wording alone. | Anthropic’s [agent-evaluation guidance](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents); Promptfoo’s [coding-agent evaluation guide](https://www.promptfoo.dev/docs/guides/evaluate-coding-agents/). | Important confound: measured outcomes belong to the model-plus-harness system unless the design isolates prompt wording. |

I found public prompt collections and repositories, including [mitsuhiko/agent-prompts](https://github.com/mitsuhiko/agent-prompts) and [system-prompts-and-models-of-ai-tools](https://github.com/x1xhlol/system-prompts-and-models-of-ai-tools). These are source material, not evidence that the prompts were validated or outperform alternatives. Leaked or reverse-engineered prompts also may be incomplete, outdated, or unauthorized; they should not be treated as canonical prompt specifications.

## 4. What the evidence does and does not support

- **Prompt wording can affect measured outcomes.** The assistant-ui and SWE-Bench Mobile sources report controlled comparisons, though each tests a narrow scope.
- **More elaborate prompts are not automatically better.** In the mobile benchmark, some detailed strategies scored below baseline; assistant-ui also reports model-dependent differences between phrasings.
- **Prompt results do not transfer automatically.** The mobile study tested one agent-model pairing, and assistant-ui’s reported results vary between Haiku and Opus.
- **System and swarm prompt effects remain unisolated for Pi and Jcode** in the evidence reviewed here. Existing benchmark scripts may compare other behaviors, but they do not establish which wording causes any observed change.
- **AgentCN-specific prompt evaluation remains unverified.** The sources reviewed describe recipes and previews; a no-results search is not proof of nonexistence.

## 5. Contrarian view and methodological risks

Prompt changes are easy to credit for effects that actually come from a different model, tool set, task distribution, sandbox, retry policy, or judge. For example, a swarm-vs-single result does not demonstrate that a swarm prompt helped unless the study holds the rest of the system constant.

Small samples can also make prompt rankings unstable. The assistant-ui README reports low trial counts for some comparisons, and SWE-Bench Mobile’s prompt ablation uses one agent-model configuration and a 50-task dataset. Its improved test-pass rate did not increase full-task success. These results support testing prompts in context, not adopting a universal “best prompt.”

## 6. Open questions

1. Have Pi maintainers or users published controlled evaluations of alternative Pi `SYSTEM.md` or `APPEND_SYSTEM.md` files?
2. Has anyone compared Jcode system or swarm prompt variants while holding model, tools, tasks, and execution limits fixed?
3. Does AgentCN publish benchmark methodology or prompt-variant results beyond its recipe descriptions and live previews?
4. How stable are the reported gains across repeated trials, model versions, repositories, and task types?
5. Which outcome matters for a future study: successful task completion, partial test coverage, cost, latency, policy adherence, or quality of the final patch?

## Sources

| Source | Use |
|---|---|
| [Pi Configuration](https://pi.dev/docs/latest/configuration) | System-prompt replacement/appending and project/user configuration. |
| [Pi Skills](https://pi.dev/docs/latest/skills) | On-demand task instructions and loading behavior. |
| [Pi Prompt Templates](https://pi.dev/docs/latest/prompt-templates) | Reusable task prompt mechanism. |
| [Pi Extensions](https://pi.dev/docs/latest/extensions) | Executable customization and prompt/context hooks. |
| [How Pi Works](https://pi.dev/docs/latest/how-pi-works) | System prompt, context, tools, skills, and agent-loop overview. |
| [Jcode prompt configuration](https://github.com/1jehuang/jcode/blob/main/docs/SYSTEM_PROMPT_CONFIG.md) | Prompt layers and override locations, inspected locally. |
| [Jcode built-in system prompt](https://github.com/1jehuang/jcode/blob/main/crates/jcode-base/src/prompt/system_prompt.md) | Default prompt source, inspected locally. |
| [Jcode swarm prompt](https://github.com/1jehuang/jcode/blob/main/crates/jcode-base/src/prompt/swarm_prompt.md) | Swarm routing guidance source, inspected locally. |
| [AgentCN repository](https://github.com/shadcn-labs/agentcn) | Agent recipe registry overview. |
| [AgentCN docs](https://www.agentcn.run/docs.md) | Recipe philosophy and supported frameworks. |
| [assistant-ui prompt-eval README](https://github.com/assistant-ui/assistant-ui/blob/main/evals/README.md) | Direct prompt A/B methodology and reported comment-hygiene results. |
| [SWE-Bench Mobile paper](https://arxiv.org/html/2602.09540) | Twelve-strategy prompt ablation and evaluation limitations. |
| [Pi Harness](https://github.com/forrestbthomas/pi-harness) | Pi-based task scorecard and reported prompt-scaffold case. |
| [Jcode discovery benchmark](https://github.com/1jehuang/jcode/blob/main/docs/DISCOVERY_BENCHMARK.md) | Bounded integration-discovery benchmark documentation. |
| [Jcode discovery call-rate benchmark](https://github.com/1jehuang/jcode/blob/main/docs/DISCOVERY_RATE_BENCHMARK.md) | Integration-tool policy benchmark documentation. |
| [Jcode swarm benchmark](https://github.com/1jehuang/jcode/blob/main/scripts/benchmark_swarm.py) | Single-agent versus swarm benchmark script. |
| [Jcode Terminal-Bench notes](https://github.com/1jehuang/jcode/blob/main/docs/TERMINAL_BENCH.md) | Jcode/Harbor benchmark setup documentation. |
| [Anthropic: Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) | Evaluation definitions, graders, trials, transcripts, and harness caveats. |
| [Promptfoo: Evaluate coding agents](https://www.promptfoo.dev/docs/guides/evaluate-coding-agents/) | Practical guidance on agent-level evaluation and architecture confounds. |
| [mitsuhiko/agent-prompts](https://github.com/mitsuhiko/agent-prompts) | Public prompt collection, not a validated evaluation. |
| [System prompts and models of AI tools](https://github.com/x1xhlol/system-prompts-and-models-of-ai-tools) | Public prompt archive, not a validated evaluation. |

## Rerun inputs

- **Workflow:** `firecrawl-deep-research`
- **Depth:** Exhaustive requested
- **Output:** Markdown matrices and cited synthesis
- **Constraint:** Research only. No evaluations run.

**Collection caveat:** Firecrawl MCP search was unavailable during this continuation. I used accessible public docs and search/fetch tools, plus prior local inspection of Jcode’s repository. The absence of an AgentCN-specific evaluation is therefore a bounded finding from sources searched, not a definitive claim.
