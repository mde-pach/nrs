---
name: nrs-fix
description: Fix NRS context gaps by analyzing reported gaps, reading relevant code, and writing or updating context files with high quality and conciseness. Use when asked to "fix gaps", "fill context gaps", "update context from gaps", or after running "nrs gap summary" and wanting to resolve reported issues.
---

# NRS Fix

Resolve context gaps reported in `nrs.gaps.md` by analyzing the codebase and writing or updating context files. Every line written must be concise, meaningful, and noise-free.

## Prerequisites

Verify `nrs` CLI is available:

```bash
nrs --help
```

## Workflow

### Step 1 — Assess

Run gap summary and read the gap file:

```bash
nrs gap summary
```

Present the gaps to the user grouped by target. Ask: *"Which area should we fix first, or should I work through all of them in priority order (most gaps first)?"*

### Step 2 — Analyze (per target area)

For each target area being fixed:

1. **Read existing context** — glob for `*.context.md` in the target directory. Read what's already there.
2. **Read relevant code** — analyze the source code in the target area to understand business concepts, patterns, and rules.
3. **Map gaps to actions** — for each gap in this target:
   - `missing-context` → create a new `domain.context.md` or `implementation.context.md`
   - `missing-concept` → add to existing `domain.context.md`
   - `missing-pattern` → add to existing `implementation.context.md`
   - `wrong` → identify and correct the inaccurate content

### Step 3 — Draft

Write the context following the writing discipline (see below). Then self-check against both writing discipline and quality checklist before presenting.

### Step 4 — Review

Present the draft to the user in a fenced code block. Ask: *"Does this accurately capture things? What should I change?"*

Iterate until the user explicitly approves. Never write without approval.

### Step 5 — Write and Validate

After approval:

1. Write the context file to disk
2. Run `nrs validate` — fix any errors iteratively
3. Run `nrs generate all` to regenerate tool entry points

### Step 6 — Clear Resolved Gaps

After all gaps for a target are resolved and validated, remove the corresponding rows from `nrs.gaps.md`. If the file is empty after cleanup, delete it.

### Step 7 — Next Target

Ask: *"Ready to move to the next area, or stop here?"* Never auto-proceed.

## Writing Discipline

Context quality determines agent effectiveness. Verbose, vague, or redundant context is worse than no context — it introduces noise that actively degrades reasoning.

### Line-Level Tests

Before keeping any line, verify:

- **Density**: remove the line — does the context lose actionable information? If not, delete it
- **One fact per line**: each bullet states exactly one thing, no compound statements
- **Code-derivable**: can this be learned by reading the code for 30 seconds? If yes, delete it

### Language Rules

- **No hedging**: delete "typically", "generally", "usually", "various", "several", "might", "can be"
- **No meta-commentary**: delete "this section covers", "the following describes", "as mentioned above"
- **Definitions over explanations**: "An order is a confirmed purchase with payment" — not "An order represents the concept of a purchase that..."
- **Rules as constraints**: "Orders cannot be modified after shipment" — not "You should avoid modifying orders after shipment"
- **Present tense, active voice**: "The scheduler runs jobs every 5 minutes" — not "Jobs are run by the scheduler"

### Draft-Level Tests

After completing a full draft:

- **Cut test**: remove 30% of the content — if meaning survives intact, the cut was right
- **Stranger test**: would a new developer understand each line without external context? If not, the line is too terse or too coupled to unstated assumptions
- **Noise audit**: read the entire draft as if it were injected into your context window alongside a coding task — does every line help you complete the task, or does some of it just sit there?

## Quality Checklist

Self-check every draft before presenting. Fix violations silently.

| Check | Applies to | Pass criteria |
|---|---|---|
| No source paths | All context files | Zero references to specific files or directories |
| Business language only | `domain.context.md` | Zero types, code snippets, or framework names |
| Patterns not listings | `implementation.context.md` | Zero file names or directory structure |
| No delegation | All context files | No "see X for details" phrases |
| Refactoring survival | All context files | Every line survives a behavior-preserving code refactor |
| Size limit | Per type | Root combined < 500 lines, domain < 300, implementation < 300 |
| No duplication | All context files | No information repeated from another context file |
| Writing discipline | All context files | Passes all line-level, language, and draft-level tests above |

## Anti-Patterns

- Writing context directly from code without abstracting to business language
- Listing files or directories instead of describing patterns
- Adding speculative content not backed by actual code or user input
- Creating `implementation.context.md` with nothing specific to say — skip it if patterns are standard
- Leaving resolved gaps in `nrs.gaps.md`
- Auto-proceeding to next target without user confirmation
