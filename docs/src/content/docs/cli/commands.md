---
title: Commands
description: NRS CLI reference.
---

## `nrs generate <target>`

Walks the project tree, generates tool entry points at every directory with `*.context.md` files.

```bash
nrs generate claude       # CLAUDE.md at every level
nrs generate all          # all registered generators
nrs generate claude --dir /path/to/project
```

| Target | Output | Ignore config | Hooks |
|---|---|---|---|
| `claude` | `CLAUDE.md` | `.claude/settings.local.json` | `.claude/settings.json` (10 hooks — see [Hooks](#hooks)) |

Ordering: nrs → corporate → team → project → domain → implementation.

---

## `nrs validate`

Checks all context files. Exit code 1 on errors, 0 on warnings.

```bash
nrs validate
nrs validate --dir /path/to/project
nrs validate --strict            # treat warnings as errors (exit 1)
```

| Check | Severity |
|---|---|
| Size limits exceeded | warning |
| Source file paths in context | error |
| Generated files out of date | error |
| Broken links in project.context.md | error |
| Reference rule violations | error |
| Duplicated content across context files | warning |
| Orphan docs not referenced from context files | warning |

---

## `nrs init`

Creates `nrs.context.md` + precommit hook.

```bash
nrs init
```

---

## `nrs install <target>`

Installs the NRS skill/config for agentic tools.

```bash
nrs install claude        # installs Claude Code skill
nrs install all           # all supported tools
```

---

## `nrs gap report`

Reports a context gap to `nrs.gaps.md`. Target is the directory the agent was working in, not a context file.

```bash
nrs gap report --type missing-context --target src/billing/ --description "no context available"
nrs gap report --type wrong --target src/orders/ --description "pricing rules outdated"
nrs gap report --type missing-concept --target src/billing/ --description "invoice lifecycle not described"
```

| Type | Agent's perspective |
|---|---|
| `missing-context` | No context was available for this area at all |
| `missing-concept` | A business concept was absent from the area's context |
| `missing-pattern` | An implementation pattern was undocumented |
| `wrong` | Something in the context didn't match reality |

Duplicates are kept — frequency signals priority. Gaps are removed manually when the underlying context is fixed.

The gap file uses a 5-column format: `Type | Target | Description | Source | Confidence`. Manual reports use `source: manual`. Automated signals use `source: observed:<pattern>`.

---

## `nrs gap summary`

Reads `nrs.gaps.md` and displays gaps grouped by target. Observed gaps show their source pattern.

```bash
nrs gap summary
nrs gap summary --dir /path/to/project
```

---

## `nrs claude observe`

Analyzes a Claude Code transcript for agent struggle signals and writes detected gaps to `nrs.gaps.candidates.md`.

```bash
nrs claude observe --transcript path/to/transcript.jsonl --dir /path/to/project
nrs claude observe --transcript path/to/transcript.jsonl --dry-run   # preview without writing
nrs claude observe --hook-mode                                        # reads hook JSON from stdin
```

Invoked automatically by the Claude Code `SubagentStop`, `Stop`, and `StopFailure` hooks installed via `nrs generate claude`. Detected gaps are staged in `nrs.gaps.candidates.md` for the `notify` command to surface.

| Pattern | Signal | Gap type | Confidence |
|---|---|---|---|
| `excessive-reads` | 5+ source files read in a directory without writing | `missing-context` or `missing-pattern` | medium/high |
| `no-context` | 3+ file operations in a directory without `*.context.md` | `missing-context` | high |
| `re-reads` | Same file read 3+ times | `missing-pattern` | medium |
| `backtracking` | Write → multiple reads → rewrite same file | `missing-pattern` | low |
| `user-correction` | User correction markers followed by file writes | `wrong` | low |

---

## `nrs claude notify`

Checks for observed context gaps and notifies the agent via hook output.

```bash
nrs claude notify --dir /path/to/project
nrs claude notify --hook-mode            # reads hook JSON from stdin
```

Invoked automatically by the Claude Code `UserPromptSubmit` hook. Reads `nrs.gaps.candidates.md`, outputs a self-contained prompt via `additionalContext` with detection metrics and triage instructions, then clears the candidates file. Silent when no candidates exist.

---

---

## `nrs claude guard`

Blocks agents from editing generated files (e.g. `CLAUDE.md`) and directs them to report a gap instead.

```bash
nrs claude guard --hook-mode   # reads hook JSON from stdin
```

Invoked automatically by the Claude Code `PreToolUse` hook (on `Edit`/`Write`). If the target file is a generated output, the edit is blocked with a message suggesting `nrs gap report`.

---

## `nrs claude layers`

Lists all CLAUDE.md files in the project with the NRS layers each contains. Used to maintain layer awareness across context boundaries.

```bash
nrs claude layers --dir /path/to/project
nrs claude layers --hook-mode      # reads hook JSON from stdin, outputs additionalContext
```

Invoked automatically by three Claude Code hooks:
- `PreCompact` — before context compaction, so compacted context retains layer paths
- `PostCompact` — after compaction, re-injecting layer paths into the new context
- `SubagentStart` — when a subagent starts, providing layer orientation

---

## Hooks

All hooks are installed by `nrs generate claude` into `.claude/settings.json`.

| Hook | Command | Purpose |
|---|---|---|
| `SessionStart` | `nrs gap summary && nrs validate` | Gap + validation briefing at session start |
| `UserPromptSubmit` | `nrs claude notify --hook-mode` | Surface observed gap candidates to the agent |
| `SubagentStop` | `nrs claude observe --hook-mode` | Signal detection on subagent transcript |
| `SubagentStart` | `nrs claude layers --hook-mode` | Layer orientation for new subagents |
| `Stop` | `nrs claude observe --hook-mode` | Signal detection on session transcript |
| `StopFailure` | `nrs claude observe --hook-mode` | Signal detection on failed session transcript |
| `PreToolUse` (Edit\|Write) | `nrs claude guard --hook-mode` | Block edits to generated files |
| `PreCompact` | `nrs claude layers --hook-mode` | Forward CLAUDE.md paths before compaction |
| `PostCompact` | `nrs claude layers --hook-mode` | Re-inject CLAUDE.md paths after compaction |
| `FileChanged` (*.context.md) | `nrs generate claude && nrs validate` | Keep generated output in sync |
