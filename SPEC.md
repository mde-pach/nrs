# NRS — Specification

NRS is an opinionated framework for agentic context and codebase organization. It structures how context is layered, referenced, and maintained in a codebase to optimize both human and AI-agent workflows. It borrows the concept of domains from Domain-Driven Design — clear boundaries with business-language context — but does not require DDD as an architecture. Every codebase has domains, whether explicitly named or not. NRS enforces strict separation of concerns across context layers.

## 1. Problem Statement

AI agents operating on codebases suffer from:

- **Context window saturation** — too many lines of context, too much noise, leading to drift and loss of track (common at 200k context, less frequent at 1M but still possible). Even with perfect retrieval, performance degrades 13.9%–85% as input length increases — length itself, not retrieval difficulty, causes degradation[2]. Information buried in the middle of context is significantly harder for LLMs to use[1].
- **Context noise** — irrelevant information in context dramatically decreases LLM reasoning performance[3]. Conversely, removing noise through compression can *improve* accuracy by up to 21.4%[4].
- **Spec drift** — implementation diverges from intent over time with no automated enforcement
- **Translation gap** — developers mentally translate between business specs and code concepts, a lossy and error-prone process
- **Coupling through context** — comments, docs, and references that create invisible dependencies between unrelated concerns. LLM coupling reasoning collapses in noisy, open-ended scenarios with F1 drops exceeding 50%[11].
- **Poor test discipline** — flaky tests, vibe-based testing, no structured process. 59% of developers encounter flaky tests monthly or more, and ~16% of tests at Google are flaky[13].
- **Agent choice ambiguity** — when agents propose multiple solutions and the user says "go", agents default to the first option without explicit consent. Agent autonomy must be a deliberate design choice, not an emergent behavior[12].
- **Unstructured AI usage can hurt** — a 2025 RCT found that AI tools *increased* experienced developer completion time by 19% on real-world tasks[9]. Meanwhile, structured repository-level documentation reduces agent runtime by ~28.6%[5]. Yet no established standard for structuring context exists — an empirical study of 466 open-source projects found considerable variation and no common content structure[22].
- **Context collapse** — without deliberate structure, context files lose detail through iterative rewriting and brevity bias[20]. An empirical study of 2,303 context files across 1,925 repositories found that developers emphasize build commands (62.3%) and implementation details (69.9%) but neglect security (14.5%) and performance (14.5%)[21].

NRS addresses these by defining a layered context system, strict reference rules, a consistent naming discipline, and a chained task workflow.

## 2. Context Layer System

Context is organized in concentric layers, from outermost (most abstract, widest scope) to innermost (most concrete, narrowest scope).

This layered approach is empirically validated: a three-tier documentation architecture tested across 283 development sessions (2,801 prompts, 16,522 agent turns) in a 108k-line codebase demonstrated that hierarchical layered docs prevent failures and maintain consistency across sessions[6]. Studies of 401 repositories with AI context files show developers are effectively building information architecture for agents[7], though 28.7% of content is duplicated when structure is ad-hoc — NRS's explicit layers prevent this.

### Layer 1 — Developer Context
- **File**: Tool-specific user config (e.g. `~/.claude/settings.json`, `~/.cursor/settings.json`). Never in the repository.
- **Scope**: Individual developer preferences
- **Examples**: Language preferences ("I speak French, answer in French, code in English"), editor settings, personal workflow habits
- **Committed**: No
- **Owner**: The individual developer

### Layer 2 — Corporate Context
- **File**: `corporate.context.md` at project root
- **Scope**: Company-wide guidelines, tools, processes
- **Examples**: Sentry for monitoring, OpenSearch for logging, Jira for tickets, Slack for communication, CI/CD standards, guided paths for company tool usage
- **Committed**: Yes, managed at the company level (not team level)
- **Owner**: Engineering leadership / platform team

### Layer 3 — Team Context
- **File**: `team.context.md` at project root
- **Scope**: Team-specific conventions and deviations from corporate standards
- **Note**: In an ideal world, this layer wouldn't exist — every team should work the same way. But reality demands it.
- **Examples**: Team-specific review processes, on-call rotations, deployment schedules
- **Committed**: Yes
- **Owner**: Team lead

### Layer 4 — Project Context
- **File**: `project.context.md` at project root
- **Scope**: Purpose, architecture, and technical presentation of the project
- **Content**: What the project does, its architectural decisions, and its dependencies on other projects (not consumers). A project that consumes telephony should reference the telephony project, but telephony should not reference its consumers — if it does, it signals a potential lack of separation of concerns.
- **Role as map**: The root `project.context.md` acts as the entry point and navigation map for the entire project. Beyond purpose and architecture, it indexes available documentation (`docs/`), skills, MCP servers, tools, and commands. An agent reading this file should know what exists and where to look without traversing the codebase.
- **Committed**: Yes
- **Owner**: Tech lead / senior developers

### Layer 5 — Domain Context
- **File**: `domain.context.md` co-located in the domain directory
- **Scope**: Business-oriented context for a specific domain within the project
- **Content**: Uses ubiquitous language, written in business terms ideally by or with product. Describes what the domain does in business terms. If natural links exist between domains, they are referenced from a business perspective, never from an implementation one. If a domain context references implementation details, it signals wrong coupling.
- **Committed**: Yes
- **Owner**: Product / domain experts, with developer review
- **Validation**: Linked documents must have their paths verified by CI/precommit hooks

### Layer 6 — Implementation Context
- **File**: `implementation.context.md` co-located in the implementation area it documents
- **Scope**: Implementation-specific architecture and patterns
- **Content**: Located directly in their concern areas. Explains repository scopes, service implementation patterns, or specific technical decisions that need documentation. This is where you describe *how* something is built, not *what* it does.
- **Committed**: Yes
- **Owner**: Developers

### Layer 7 — Code and Inline Documentation
- **File**: The source files themselves
- **Scope**: The actual source code and its inline documentation
- **Content**: The code itself. Inline documentation should explain *why*, not *what* (the code shows *what*).
- **Committed**: Yes
- **Owner**: Developers

## 2.1. File Structure

Context files are co-located with the code they describe. The layer is encoded in the filename — no frontmatter needed, CI can validate by name alone.

```
project/
├── CLAUDE.md                              # generated — DO NOT EDIT
├── .cursorrules                           # generated — DO NOT EDIT
├── corporate.context.md                   # Layer 2 — company-wide guidelines
├── team.context.md                        # Layer 3 — team conventions
├── project.context.md                             # Layer 4 — project map & architecture
├── nrs.gaps.md                            # gap reports — agent-reported context issues
├── docs/                                  # on-demand documentation
│   ├── billing/
│   │   ├── pricing-rules.md               # pricing model and discount logic
│   │   └── invoice-lifecycle.md           # invoice states and transitions
│   ├── shipping/
│   │   └── carrier-selection.md           # carrier routing rules
│   ├── testing.md                         # test framework, setup, patterns
│   ├── server-components.md               # rendering strategy
│   └── ...
├── src/
│   ├── billing/
│   │   ├── domain.context.md              # Layer 5 — billing domain (business)
│   │   ├── repositories/
│   │   │   ├── implementation.context.md  # Layer 6 — repository patterns
│   │   │   └── ...
│   │   ├── services/
│   │   │   ├── implementation.context.md  # Layer 6 — service patterns
│   │   │   └── ...
│   │   └── ...
│   ├── telephony/
│   │   ├── domain.context.md              # Layer 5 — telephony domain (business)
│   │   └── ...
│   └── ...
└── ...
```

### Naming Convention

| Layer | Filename | Location |
|---|---|---|
| 1 — Developer | Tool-specific config | Outside repository |
| 2 — Corporate | `corporate.context.md` | Project root |
| 3 — Team | `team.context.md` | Project root |
| 4 — Project | `project.context.md` | Project root |
| 5 — Domain | `domain.context.md` | Domain directory |
| 6 — Implementation | `implementation.context.md` | Implementation area |
| — NRS rules | `nrs.context.md` | Project root |
| — Gap reports | `nrs.gaps.md` | Project root |
| 7 — Code | Source files | Everywhere |

### Generated Tool Entry Points

Tool-specific files (`CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `GEMINI.md`, etc.) are **generated** from the root-level context files (layers 2–4). They are never edited by hand.

Each generated file:
1. Carries a `DO NOT EDIT — generated by NRS` header
2. Aggregates content from `nrs.context.md`, `corporate.context.md`, `team.context.md`, and `project.context.md`
3. Is formatted for its target tool's expected syntax
4. Is regenerated by a precommit hook on every commit

Domain and implementation context files (layers 5–6) are **not** included in generated tool files. They are discovered on-demand by the agent during work — this is the on-demand loading principle from §3.

### Discovery

Agents discover deeper context files by globbing for `*.context.md` within the directory they are working in. This pattern is tool-agnostic and works with any agent that supports file search.

### Documentation (`docs/`)

The `docs/` directory at project root holds detailed, on-demand documentation that goes deeper than context files can or should. Context files describe *what* and *why* at a high level; docs explain *how* in detail.

Docs are:
- **Referenceable from any context file**: Any `*.context.md` may link to docs with markdown links. A domain context may link to a doc for deeper business process documentation, an implementation context may link to a doc for detailed patterns.
- **On-demand loaded**: Like cold-memory context, docs are only read when the current task requires them.
- **Folder structure reflects abstractions**: Organize docs into shallow folders that mirror the project's conceptual structure — by domain, by concern, by workflow, whatever the project's natural boundaries are. Agents navigate folders using file-system tools; a meaningful hierarchy makes docs discoverable without scanning a flat list[23].
- **Topic-scoped**: Each document covers one topic completely. One doc per topic, not one doc per layer.
- **Subject to the same anti-coupling rules**: Docs describe patterns and knowledge, not file inventories. They must survive refactoring.
- **Not context files**: Docs are regular markdown files, not `*.context.md`. They don't participate in reference rules or size limits — but they should still be as concise as the topic allows. A doc loaded into agent context has the same degradation effects as any other content[2].
- **Illustrative code allowed**: Docs may include code examples, boilerplates, and pattern snippets. These are illustrative — they teach a pattern, not reference specific project files — and do not create coupling.

## 3. Context File Constraints

These are enforceable thresholds derived from empirical research on LLM performance under varying context conditions.

### Size Limits

| File | Max lines | Rationale |
|---|---|---|
| `CONTEXT.md` + `corporate.context.md` + `team.context.md` (combined, always-loaded) | ~500 lines | Vasilopoulos's "constitution" kept to ~660 lines for a 108k-line codebase[6] |
| `domain.context.md` | ~300 lines | Agent spec sweet spot: 115–1,233 lines, median ~300–700[6] |
| `implementation.context.md` | ~300 lines | Same rationale; on-demand loaded, should be focused on its specific concern |

These are upper bounds, not targets. Shorter is better — 4x compression of prompts *improved* accuracy by 21.4%[4].

### Multi-Instance Reasoning Limitation

There is no hard cap on the number of context files loaded simultaneously — every file the task needs must be available. However, LLMs exhibit a fundamental limitation when aggregating information across multiple sources: performance degrades as instance count increases, independently of total context length, and without warning to the user[16]. This degradation is not about noise — it occurs even when all instances are relevant.

This has a direct implication for NRS: when a task requires reasoning across many context files simultaneously, it should be decomposed through sub-agents (§9) that each process a subset and produce a formalized analysis, rather than expecting a single agent to aggregate across all sources at once.

### Information Placement

Critical information MUST be placed at the **beginning or end** of context files, never buried in the middle. LLMs exhibit a U-shaped attention curve — performance is highest for information at the start and end of input, and drops significantly for content in the middle[1]. This holds even for models designed for long contexts.

In practice, this means each context file should:
1. Open with the most important constraints and rules
2. Place detailed but less critical content in the body
3. Close with key reminders or decision summaries

### Density Over Verbosity

Every line in a context file must carry unique, non-duplicative information. The optimization target is not token minimization but **semantic density** — the ratio of meaningful information to total tokens[17]. Tokens carrying high semantic value (descriptive names, business logic, constraints) are investments that reduce downstream reasoning costs. Tokens carrying zero information (boilerplate, ceremony, redundant context) are waste. Critically, compressing high-information tokens is counterproductive: a controlled experiment showed aggressive compression increased total session cost by 67% despite reducing input tokens by 17%, because it shifted interpretive burden to the model's reasoning phase[17].

Context files MUST NOT:
- Duplicate information already present in code or other context files (28.7% duplication is the norm without discipline[7])
- Include boilerplate, templates, or filler text
- Restate what can be inferred from the code itself

### Anti-Coupling Between Context and Code

Context files must be resilient to implementation changes. A context document should only need updating when its own layer's concerns change — never because code was refactored underneath it.

**No source file path references.** Context files MUST NOT reference specific source files, directories, or code paths. If a file is renamed, moved, or split, no context document should break. Agents discover context files via glob patterns, not hardcoded paths. Same-level contexts MAY reference each other by domain name to state facts (e.g., "Orders snapshot product price at purchase time"), but must not delegate understanding (e.g., "see the Orders domain context for how orders work").

**Domain context describes business, not implementation.** A `domain.context.md` must survive any refactoring that preserves business behavior. It describes concepts ("a product has a price") not types (`priceInCents: number`). It describes rules ("price changes don't affect existing orders") not implementations. If a domain property is renamed in code but the business concept stays the same, the domain context MUST NOT need updating.

**Implementation context describes patterns, not inventories.** An `implementation.context.md` describes architectural decisions and patterns ("services are stateless functions, dependencies are injected") not file listings. Files change constantly — patterns change rarely. If the context reads like a directory listing, it's coupled to the code and will rot.

**Test: would a refactor break this?** Before writing a line in any context file, ask: if the code under this file were refactored without changing behavior, would this line need updating? If yes, it doesn't belong in context — it belongs in the code itself (as inline documentation or as the code structure).

### On-Demand Loading

Context files beyond the always-loaded root context MUST be loaded on-demand, only when the current task requires them. This follows the validated three-tier pattern[6]: hot memory (always loaded, small), domain specialists (invoked per task), cold memory (loaded on explicit need). A large portion of performance degradation occurs within the first 7,000 tokens of added context[2].

## 4. Reference Rules

These rules prevent invisible coupling between concerns. Research shows that LLM reasoning about coupling collapses in noisy, open-ended scenarios (F1 drops >50%), and cohesion analysis fails without structured guidance[11]. Explicit reference rules provide that guidance.

1. **Same level**: A context document MAY reference other documents at the same level (e.g., one domain context referencing another domain)
2. **Never higher**: A context document MUST NOT reference a layer above it (outer/more abstract). Domain context must not reference corporate processes.
3. **Should not lower**: A context document SHOULD NOT reference a layer below it (inner/more concrete). Domain context should not reference implementation patterns. If it does, it signals wrong coupling.

Violations of these rules are architectural smells and should be caught in review or automated checks.

## 5. Domain Thinking

NRS borrows the concept of domains from Domain-Driven Design: clear boundaries with business-language context. Every codebase has domains — distinct areas of business logic with their own concepts and rules — whether the project follows DDD or not. NRS requires clear domain boundaries, not a specific architecture.

The core value: **unifying spec and code naming**. When a business concept is called "Subscription" in the spec, it should be called `Subscription` in the code. This removes the translation step that developers otherwise perform mentally — a step that is lossy, inconsistent, and invisible.

In NRS this means:
- **Clear domain boundaries**: Every project defines its domains, each with a `domain.context.md` written in business terms
- **Ubiquitous language**: One vocabulary shared between product, specs, and code — as much as the codebase allows
- **Domain context in business terms**: Written by or with product where possible, describing what the domain does, not how it's implemented
- **Code structure reflects domains**: The codebase navigation should follow domain boundaries where possible

## 6. Testing Philosophy

Tests are the **only automated mechanism** that ensures specs are met. They are central, not secondary. Industrial studies show TDD reduces pre-release defect density by 40–90%[14].

### Principles

- **Tests are spec enforcement**: A test proves a spec holds. If there's no test, the spec is unverified.
- **Zero flakiness tolerance**: A test must deterministically pass or fail. If a bug occurs 1 time in 10,000, the test must run 10,000+ entries to trigger it. A test that sometimes passes and sometimes fails is a test that does not capture what it claims to test. This is not pedantic: ~16% of tests at Google are flaky, and 1 in 11 GitHub commits had a red build from flaky tests in 2020[13].
- **Strict process, not vibes**: Test design follows a structured process. The historical tendency to treat tests as secondary ("vibes testing") is rejected. Good practices exist and must be followed. The most common root causes of flaky tests — async waits, concurrency issues, and test order dependency[13] — are all preventable with disciplined design.
- **Human testing remains relevant**: Automated tests do not replace human testing. Human test plans can be inferred from implementation and discussed with agents.

### Evidence-Based Bug Approach

When a bug is identified:
1. Write a test that highlights the issue (evidence-based)
2. Invest in the test until the bug is reliably reflected in it
3. Fix the bug
4. Ensure the test passes

The test comes first. The fix comes second. This guarantees the bug is captured and won't regress.

## 7. Chained Task Workflow

Development follows a chained dependency system that leverages the task system. Every step depends on the previous one completing successfully.

### Feature Flow
1. Ticket enablement
2. Plan the feature (interaction/discussion)
3. Implement the code
4. Add e2e tests
5. Add integration tests
6. Implement remaining code
7. Ensure tests pass
8. Commit
9. Check: does NRS context need updating? If yes → update context docs
10. Report any context gaps encountered during the task
11. Run full e2e suite

### Bug Fix Flow
1. Ticket enablement
2. Identify the bug (evidence-based approach)
3. Write a test highlighting the issue
4. Invest in the test until the bug is reflected
5. Fix the bug
6. Ensure the test passes
7. Commit
8. Check: does NRS context need updating? If yes → update context docs
9. Report any context gaps encountered during the task
10. Run full e2e suite

### Change Feature Flow
1. Ticket enablement
2. Add/change integration tests
3. Implement the code
4. Ensure tests pass
5. Follow commit and doc update steps as above
6. Report any context gaps encountered during the task

### Gap Reporting

During development, agents discover that their available context is missing information or contains incorrect information. Gap reporting provides a lightweight telemetry mechanism to surface these issues without disrupting the development workflow. Agents do not interact with context files directly — they report gaps in terms of the working directory and what was missing. The mapping from gap to specific context file is a resolution concern, not a reporting concern.

#### Gap Types

| Type | Agent's perspective |
|---|---|
| `missing-context` | No context was available for this area at all |
| `missing-concept` | A business concept was absent from the area's context |
| `missing-pattern` | An implementation pattern was undocumented |
| `wrong` | Something in the context didn't match reality |

#### Mechanism

- Gaps are reported via `nrs gap report --type <type> --target <directory> --description "text"`
- The target is the directory the agent was working in, not a context file
- Reports are appended to `nrs.gaps.md` at the project root
- The file is committed to git — gaps are shared state visible to all developers and agents
- Duplicate reports are intentional: frequency indicates priority
- There is no resolve command — gaps are removed manually in the same commit that fixes the underlying context

#### When to Report

An agent reports a gap when it needed to read source files beyond those it was modifying in order to understand the working area — its domain concepts, implementation patterns, or architecture. Reading source files to modify them is normal work; reading source files to build a mental model that context should have provided is a gap.

Gaps are reported after task completion, not during. Only report when gaps exist — silence means success.

## 8. General Principles

### Minimize
- **Context lines**: Less context = less noise = better agent performance
- **Context pollution**: Every line in context must earn its place
- **Tasks per goal**: Fewer steps to achieve an outcome
- **Linting work**: Formatters run on precommit. Developers should never manually fix formatting.

### Ensure
- **No drift from spec**: Automated checks keep implementation aligned with intent
- **Auto-updated docs**: Documentation updates are part of the workflow, not an afterthought. Automation is the only reliable prevention of outdated, incomplete, and inconsistent information.
- **Codebase navigability from higher-level files**: You should not need to read the entire codebase to make a change. The structure is carried by context documents at higher layers. SWE-bench demonstrated that codebase structure and organization are the critical barriers for AI agents tackling real-world issues[10].
- **Context gap feedback loop**: Agents report missing or incorrect context during development via `nrs gap report`. Frequency of reports signals priority — the most-reported gaps get fixed first. Over time, context files converge toward precisely sufficient coverage driven by real usage, not speculative completeness.

### Avoid
- **Loop-block work**: Development should not get stuck in unproductive loops
- **Comments driving coupling**: A comment that references another module creates an invisible dependency
- **Comments carrying changes**: When implementation changes, the change should be in the implementation, not scattered across comments in other files
- **Requiring full codebase traversal**: The context layer system exists so that you can navigate top-down from high-level docs to the specific area you need

## 9. Agent-Specific Guidelines

### Sub-Agent Strategy
Large codebases produce better results when using intermediary sub-agents to formalize and analyze rather than dumping raw code paths into context. The sub-agent produces a structured analysis; the main agent works from that analysis, not from the code directly. Multi-agent architectures with focused roles outperform monolithic approaches — achieving 96.3% vs 90.2% accuracy while using 60% fewer tokens[8]. Coding agents that organize information in file systems and use tools outperform standard long-context approaches by 17.3% on average, handling corpora up to three trillion tokens[18].

However, multi-agent decomposition is not free of risk. 75.3% of multi-agent failures stem from information degradation during the planning-to-coding handoff — semantically equivalent inputs cause 7.9%–83.3% failure rates when plans are poorly structured[19]. Clear, structured context at each stage directly mitigates this.

### Context Survival
The task-based approach is designed to survive context window compaction. Tasks persist across conversation compression, maintaining continuity even when earlier messages are compacted.

### Output Discipline
Verbose tool outputs (test results, build logs, lint reports) must be filtered at the point of ingestion — only actionable information should persist in working context. The same separation that sub-agents provide at the architecture level applies within a single session. Carrying full traces forward degrades reasoning even when the information remains retrievable[1].

### Propose First, Act After
Every significant action must be proposed and approved before execution. This applies to implementation plans, architectural decisions, approach choices, and any work that consumes time or changes code. The agent presents what it intends to do, the user confirms, then work begins. This minimizes wasted iterations and keeps the user in control of direction. Agent autonomy must be a deliberate, calibrated design decision[12].

### Precommit Automation
- Formatters run automatically
- Gitmoji convention onboarded via precommit
- Context document path validation via CI/precommit hooks
- Cross-file duplication detection — warns when content blocks are repeated across context files (28.7% duplication is the norm without discipline[7])
- Gap file (`nrs.gaps.md`) committed alongside context fixes — CI can surface open gap count as a maintenance signal

## 10. What NRS is NOT

- **Not a library**: NRS does not ship runtime code. It is a structural and organizational framework.
- **Not technology-specific**: NRS applies to any codebase, though it may favor certain implementations over others.
- **Not architecture-specific**: NRS does not require DDD or any specific architecture. It requires clear domain boundaries with business-language context — a concept borrowed from DDD, applicable to any codebase.

___

[1]: https://arxiv.org/abs/2307.03172 (Liu, N.F. et al. "Lost in the Middle: How Language Models Use Long Contexts." TACL, 2024.)
[2]: https://arxiv.org/abs/2510.05381 (Du, Y. et al. "Context Length Alone Hurts LLM Performance Despite Perfect Retrieval." EMNLP, 2025.)
[3]: https://arxiv.org/abs/2302.00093 (Shi, F. et al. "Large Language Models Can Be Easily Distracted by Irrelevant Context." ICML, 2023.)
[4]: https://arxiv.org/abs/2310.06839 (Jiang, H. et al. "LongLLMLingua: Accelerating and Enhancing LLMs in Long Context Scenarios via Prompt Compression." ACL, 2024.)
[5]: https://arxiv.org/abs/2601.20404 (Lulla, J.L. et al. "On the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents." 2026.)
[6]: https://arxiv.org/abs/2602.20478 (Vasilopoulos, A. "Codified Context: Infrastructure for AI Agents in a Complex Codebase." 2026.)
[7]: https://arxiv.org/abs/2512.18925 (Jiang, S. & Nam, D. "Beyond the Prompt: An Empirical Study of Cursor Rules." MSR, 2026.)
[8]: https://arxiv.org/abs/2312.13010 (Huang, D. et al. "AgentCoder: Multi-Agent-based Code Generation with Iterative Testing and Optimisation." 2024.)
[9]: https://arxiv.org/abs/2507.09089 (Becker, J. et al. "Measuring the Impact of Early-2025 AI on Experienced Open-Source Developer Productivity." 2025.)
[10]: https://arxiv.org/abs/2310.06770 (Jimenez, C.E. et al. "SWE-bench: Can Language Models Resolve Real-World GitHub Issues?" ICLR, 2024.)
[11]: https://arxiv.org/abs/2511.20933 (Saad, M. et al. "Hierarchical Evaluation of Software Design Capabilities of LLMs." 2025.)
[12]: https://arxiv.org/abs/2506.12469 (Feng, K.J.K. et al. "Levels of Autonomy for AI Agents." 2025.)
[13]: https://dl.acm.org/doi/10.1145/3476105 (Parry, O. et al. "A Survey of Flaky Tests." ACM TOSEM, 2022.)
[14]: https://link.springer.com/article/10.1007/s10664-008-9062-z (Nagappan, N. et al. "Realizing Quality Improvement Through Test Driven Development." Empirical Software Engineering, 2008.)
[16]: https://arxiv.org/abs/2603.22608 (Chen et al. "Understanding LLM Performance Degradation in Multi-Instance Processing." 2026. Instance count stronger effect than context length.)
[17]: https://arxiv.org/abs/2604.07502 (Ustynov, D. "Beyond Human-Readable: Rethinking Software Engineering Conventions for the Agentic Development Era." 2026.)
[18]: https://arxiv.org/abs/2603.20432 (Cao, W. et al. "Coding Agents are Effective Long-Context Processors." 2026.)
[19]: https://arxiv.org/abs/2510.10460 (Lyu, Z. et al. "Understanding and Bridging the Planner-Coder Gap." 2025.)
[20]: https://arxiv.org/abs/2510.04618 (Zhang, Q. et al. "Agentic Context Engineering: Evolving Contexts for Self-Improving Language Models." 2025.)
[21]: https://arxiv.org/abs/2511.12884 (Chatlatanagulchai, W. et al. "Agent READMEs: An Empirical Study of Context Files for Agentic Coding." 2025.)
[22]: https://arxiv.org/abs/2510.21413 (Mohsenimofidi, S. et al. "Context Engineering for AI Agents in Open-Source Software." 2025.)
[23]: https://arxiv.org/abs/2603.20432 (Cao, J. et al. "Coding Agents with File-System Tools." 2026.)
