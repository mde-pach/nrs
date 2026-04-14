# NRS Context

NRS (Nested Reference System) provides structured context to guide AI agents working in this codebase.

## Gap Reporting

After completing a task, if you needed to read source files beyond those you modified to understand the working area — its domain concepts, implementation patterns, or architecture — report each gap:

    nrs gap report --type <type> --target <directory> --description "what's missing or wrong"

Target is the directory you were working in, not a specific file. Types:
- `missing-context` — no context was available for this area at all
- `missing-concept` — a business concept was absent from the area's context
- `missing-pattern` — an implementation pattern was undocumented
- `wrong` — something in the context contradicts reality

Report only — never modify context as part of gap reporting.

## Propose First, Act After

Every significant action — implementation plans, architectural decisions, approach choices — must be proposed and approved before execution.

## Testing

Context defines the spec. Tests verify it. No test means the spec is unverified.

- Write a failing test before fixing any bug
- Tests must be deterministic — if an issue occurs 1 in 10,000 times, run 10,000+ entries
- Prefer real implementations over test doubles — mocks assert how code calls a dependency, not whether the system works
- Test-double hierarchy: real → fake → stub → spy → mock (last resort)
- Integration and e2e over unit tests — hit real infrastructure

## Sub-Agent Strategy

When a task touches more than one domain or requires reading more than 5 files to understand the area, use sub-agents.

- Each sub-agent is scoped to one domain, one file group, or one analysis question
- Sub-agents produce structured analyses — the main agent works from analyses, never from raw code
- The main agent's context is for decision-making and synthesis, not data processing
- Use sub-agents for *information gathering* across multiple sources; use single-agent for *implementation* tasks requiring consistent decision-making
- Avoid parallel sub-agents for tasks with shared state — inter-agent misalignment is a primary multi-agent failure mode

## Output Discipline

When processing verbose tool outputs (test results, build logs, lint reports), extract only actionable information. Do not preserve full traces in working context. Sub-agents already provide this filtering at the architecture level — apply the same principle within a single session.

## Signals

NRS hooks observe agent behavior and automatically report context gaps. Gaps from signals appear in `nrs.gaps.md` with source `observed:<pattern>`. Manual gap reporting continues to work alongside automated signals.

Hook lifecycle: SessionStart (gap + validation briefing) → SubagentStart (layer orientation) → PreToolUse (guard generated files) → SubagentStop/SessionEnd (observe transcript) → TaskCompleted (notify about gaps) → PreCompact/PostCompact (forward layer paths) → FileChanged (sync generated output).

## Commands

- `nrs gap report` — report a context gap
- `nrs gap summary` — view reported gaps grouped by target
