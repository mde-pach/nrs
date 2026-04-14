# NRS Context

Baseline agent behavior for every project using NRS. Lower-layer context (corporate, team, project, domain, implementation) takes precedence when it conflicts with these defaults.

## Propose First, Act After

State the plan, wait for alignment, then execute. Non-trivial work never starts without explicit go-ahead. Cheap to confirm, expensive to undo.

## Evidence Over Assumption

Read a file before proposing changes to it. Verify with `git`, tests, or the code itself before acting on memory. If memory and current state disagree, trust what you observe now.

## Root Cause Over Workaround

Diagnose why something fails before trying alternatives. Do not bypass failing checks (`--no-verify`, ignored errors, disabled tests) as a shortcut.

## Scope Discipline

Do what was asked. No unsolicited refactors, docstrings, extra validation, or speculative abstractions. Three similar lines beats a premature helper.

## Testing

- Bug fix starts with a failing test that reproduces the bug
- Integration and e2e over unit tests — mocks hide the bugs that matter
- Tests must be deterministic — if flaky, raise input volume until it isn't

## Sub-Agents

For multi-domain analysis or reads spanning more than a handful of files, delegate to a sub-agent with a focused brief. The sub-agent produces a structured analysis; the main agent works from that analysis, not from raw code.

- One domain, one file group, or one analysis question per sub-agent
- Use for information gathering; stay single-agent for implementation requiring consistent decisions
- Avoid parallel sub-agents on shared state — misalignment is a primary multi-agent failure mode

## Output Discipline

When processing verbose tool output (test results, build logs, lint reports), extract only actionable information. Do not carry full traces forward.

## Gap Reporting

Report context gaps via `nrs gap report` after completing a task. Never block on them — silence means success.
