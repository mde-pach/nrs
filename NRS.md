# NRS

Framework for agentic context and codebase organization. Built on Domain-Driven Design. Every rule below exists to reduce context noise, prevent coupling, and keep agents effective.

## Context Layers

Seven concentric layers, outermost to innermost. Each layer can reference same-level peers, MUST NOT reference higher (outer) layers, SHOULD NOT reference lower (inner) layers.

| Layer | File | Location | Owner |
|---|---|---|---|
| 1. Developer | Tool user config | Outside repo | Developer |
| 2. Corporate | `corporate.context.md` | Project root | Engineering leadership |
| 3. Team | `team.context.md` | Project root | Team lead |
| — NRS | `nrs.context.md` | Project root | — |
| 4. Project | `project.context.md` | Project root | Tech lead |
| 5. Domain | `domain.context.md` | Domain directory | Product / domain experts |
| 6. Implementation | `implementation.context.md` | Implementation area | Developers |
| 7. Code | Source files | Everywhere | Developers |

## Root Context as Map

`project.context.md` is the entry point and navigation map. It indexes: project purpose, architecture, domains, documentation (as markdown links to `docs/`), tools, commands, skills, and MCP servers. An agent reading this file knows what exists and where to look.

## Context File Rules

**Size limits.** Root contexts combined (layers 2–4): ~500 lines max. Individual `domain.context.md` or `implementation.context.md`: ~300 lines max. These are ceilings, not targets — shorter is always better.

**Density.** Every line must carry unique information. No duplication of what's in code or other context files. No boilerplate. No filler.

**Placement.** Critical information at the beginning and end of files, never buried in the middle.

**On-demand loading.** Only root contexts (layers 2–4) are always loaded. Domain and implementation contexts are loaded when the agent enters their area. Documentation is loaded when the task requires it.

**Standalone.** Each context file must make sense on its own without knowing other context files exist.

## Anti-Coupling

Context files must survive refactoring that preserves behavior.

- **No source file paths.** Never reference specific source files or directories. Agents discover files via glob patterns, not hardcoded paths. The only allowed path references are markdown links in the root `project.context.md` map (to `docs/`).
- **Domain context = business language.** Describes concepts ("a product has a price") not types (`priceInCents: number`). Describes rules, not implementations. If a property is renamed in code but the concept stays the same, the domain context must not need updating.
- **Implementation context = patterns, not inventories.** Describes decisions and conventions ("services are stateless, dependencies are injected") not file listings. Only create one when there is something specific to say — if the pattern is the same as the parent level, no file is needed.
- **Refactoring test.** Before writing a line: if the code under this file were refactored without changing behavior, would this line break? If yes, it belongs in the code, not in context.

## Documentation (`docs/`)

The `docs/` directory holds on-demand documentation deeper than context files. Any `*.context.md` may link to docs with markdown links.

- Topic-scoped: one document per topic, not per layer or domain
- Same anti-coupling rules apply: describe patterns, not file inventories
- Illustrative code, boilerplates, and pattern snippets are allowed — they teach patterns without creating coupling
- No hard size limit, but conciseness still matters — loaded docs have the same context degradation effects as any content

## Domain-Driven Thinking

Clear domain boundaries with business-oriented context are required. Full DDD implementation is strongly recommended but not mandatory — what matters is that every project defines clear domains with business-language context.

- Ubiquitous language: one vocabulary shared between product, specs, and code where possible
- Clear domain boundaries: each domain has a `domain.context.md` in business terms
- Domain context written by or with product, not developers alone
- Code structure reflects domain boundaries

## Testing

Tests are the only automated proof that specs hold.

- **Zero flakiness.** A test deterministically passes or fails. If an issue occurs 1 in 10,000 times, the test runs 10,000+ entries.
- **Evidence-based bugs.** Write a failing test first, invest until the bug is reflected, then fix. Test before fix, always.
- **Strict process.** Test design follows structured conventions documented in `docs/`. No vibes.

## Development Workflow

Chained dependency system — each step depends on the previous completing.

**Feature:** ticket → plan → implement → add e2e tests → add integration tests → implement remaining → ensure passing → commit → update NRS context if needed → run full e2e suite

**Bug fix:** ticket → identify bug → write failing test → invest in test → fix → ensure passing → commit → update context if needed → run e2e suite

**Change feature:** ticket → add/change integration tests → implement → ensure passing → commit → update context if needed → run e2e suite

## Agent Guidelines

- **Propose first, act after.** Every significant action must be proposed and approved before execution. Present the plan, get confirmation, then work. This minimizes wasted iterations.
- **Sub-agents over monolithic.** When reasoning across many sources, decompose through sub-agents that each process a subset and produce a formalized analysis.
- **Context survival.** Use task-based approaches that survive context window compaction.
- **Discovery.** When entering a directory, glob for `*.context.md` to find relevant context.

## Generated Tool Entry Points

Tool-specific files (`CLAUDE.md`, `.cursorrules`, `.windsurfrules`, `GEMINI.md`) are generated from layers 2–4. They carry a `DO NOT EDIT — generated by NRS` header and are regenerated on every commit. Never edit them by hand.

## Precommit Automation

- Formatters run automatically
- Gitmoji convention
- Context document link validation
- Generated tool files regenerated
