---
name: nrs-setup
description: Set up NRS (Nested Reference System) in an existing project. Use when asked to "set up NRS", "initialize NRS", "add NRS to this project", "create context files", or when working with a project that needs agentic context organization. Handles creating nrs.context.md, project.context.md, domain contexts, implementation contexts, docs/, and running the NRS CLI for generation and validation.
---

# NRS Setup

Set up NRS context organization in an existing project through a progressive, interactive workflow. Each context document is crafted one at a time with the user, ensuring quality at every step.

## Prerequisites

Verify `nrs` CLI is available:

```bash
nrs --help
```

If missing, install from source:

```bash
git clone https://github.com/maximedepachtere/nrs.git /tmp/nrs-install
cd /tmp/nrs-install/cli && cargo install --path .
```

## Core Principle: Single-Document Lifecycle

Every context document follows this loop. Never skip steps.

1. **Gather** — Ask targeted questions, never more than 3 at a time. State what you already know from prior exploration and ask only for gaps.
2. **Draft** — Write the complete document content.
3. **Self-Check** — Before presenting, silently verify against the quality checklist (see below). Fix any violations.
4. **Present** — Show the full draft to the user in a fenced code block with the filename as header. Ask: *"Does this accurately capture things? What should I change?"*
5. **Iterate** — If the user gives feedback, revise and re-present. Repeat until the user explicitly approves.
6. **Write** — Only after approval, write the file to disk.
7. **Transition** — Explain what comes next. Ask if the user wants to continue. Never auto-proceed to the next document.

## Workflow

### Phase 0 — Orient

**Goal:** Build shared understanding before touching anything.

1. Read the project README and any existing documentation (package.json, pyproject.toml, Makefile, config files, existing docs/).
2. Explore the codebase directory structure (top 2 levels).
3. Present your understanding back to the user in 5-8 bullet points covering:
   - What the project does
   - Tech stack identified
   - Potential domain areas spotted
   - Commands and tooling found
   - Documentation that already exists
4. Ask: *"Is this understanding correct? What did I miss or get wrong?"*
5. Iterate until the user confirms your understanding is accurate.

Do not proceed until the user confirms. A wrong assumption here will propagate into every document.

### Phase 1 — Initialize

```bash
nrs init
```

Creates `nrs.context.md` (agent operating rules) and the precommit hook. This is a standard template — no customization needed. Confirm to the user what was created and move on.

### Phase 2 — Project Context (`project.context.md`)

This is the most critical document — the project map that every agent reads first. Take your time.

**Round 1 — Purpose & Architecture:**
- State what you learned from the README about the project's purpose and architecture.
- Ask: *"What would you correct or add about the project's purpose? Are there architectural components I missed?"*

**Round 2 — Domains & Key Decisions:**
- Propose a domain list based on your codebase exploration: *"I see these potential domain areas: [list]. Does this match how you think about the codebase?"*
- Ask: *"Are there key technical decisions that any developer working here needs to know?"*

**Round 3 — Tools & Documentation:**
- List commands you found in package.json, Makefile, or similar.
- Ask: *"Are there other commands or tools I should include? What documentation topics would be valuable for the docs/ section?"*

After the 3 rounds, draft `project.context.md` following the format in [references/context-formats.md](references/context-formats.md). Follow the single-document lifecycle: self-check, present, iterate, approve.

After approval, create the `docs/` directory and empty placeholder files for each documentation entry referenced in the file. Do not write doc content yet — that comes in Phase 6.

### Phase 3 — Corporate and Team Contexts (gated, optional)

**Corporate gate:** Ask the user: *"Does your organization have company-wide engineering standards that should apply to this project? (e.g., mandated tools, coding standards, CI/CD requirements, observability rules). If not, we'll skip this."*

If yes:
- Ask 2-3 targeted questions about the standards.
- Follow the single-document lifecycle for `corporate.context.md`.

If no: skip and move on.

**Team gate:** Ask the user: *"Does your team have conventions that differ from corporate standards, or team-specific practices? (e.g., different review process, deployment strategy, local tooling). If not, we'll skip this."*

If yes:
- Ask about scope and deviations from corporate.
- Follow the single-document lifecycle for `team.context.md`.

If no: skip and move on.

### Phase 4 — Domain Contexts (one at a time, prioritized)

**Entry gate:** Present the domain list from `project.context.md`. Ask: *"Let's work through these one at a time. Which domain is the most important to get right first?"*

**For each domain, follow the single-document lifecycle with these gathering questions (one at a time, not all at once):**

1. *"In business terms, what does [Domain] do? Imagine explaining it to a product person who has never seen the code."*
2. *"What are the core business concepts in this domain? For each, give me a one-sentence business definition."*
3. *"What business rules govern this domain — things that must always be true?"*
4. *"How does this domain relate to the other domains from a business perspective?"*

Draft `domain.context.md` in the domain's directory. Self-check, present, iterate, approve.

**After each domain:** Ask: *"Ready to move to the next domain, or would you like to stop here for now?"* Never auto-proceed.

**Push back on anti-patterns:** If the user provides types, code, framework names, or file paths in their answers, redirect: *"That sounds like implementation detail. How would you describe this in pure business terms, without referencing the code?"*

### Phase 5 — Implementation Contexts (selective, gated)

Implementation contexts are opt-in. Most areas don't need one.

**Per area, gate question:** *"Is there something specific and non-obvious about how [Area] is built that a developer needs to know to work effectively here? If the patterns are standard, we skip this."*

If yes:
- Ask: *"What patterns, conventions, or architectural decisions are specific to this area? Not what files exist — what rules a developer must follow."*
- Follow the single-document lifecycle for `implementation.context.md`.

If no: skip and move to the next area.

**Push back on anti-patterns:** If the user describes file structure or lists filenames, redirect: *"That describes structure rather than a pattern. Could you rephrase as a convention or decision? For example, instead of 'handlers are in /api/handlers/', say 'each API endpoint is a standalone handler function that receives a validated request DTO.'"*

### Phase 6 — Documentation (`docs/` files, deferrable)

Ask: *"Would you like to write the documentation files now, or come back to them later?"*

If now: for each `docs/` placeholder created in Phase 2, ask what it should cover, draft the content, present, iterate, approve.

If later: note the placeholder files and move on.

### Phase 7 — Generate and Validate

```bash
nrs generate claude
nrs validate
```

If validation errors:
- Explain each error clearly to the user.
- Propose fixes.
- Apply fixes after user agreement.
- Re-run validation until clean.

Present a completion summary:
- Every file created, with its purpose.
- What was skipped and why.
- Reminder: *"Update context files when business concepts or patterns change, not when code is refactored."*

## Writing Discipline

Context quality determines agent effectiveness. Verbose, vague, or redundant context is worse than no context — it introduces noise that actively degrades reasoning. Apply these rules during drafting, not after.

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

Self-check every draft against this table before presenting it. Fix violations silently — the user should see a clean draft on first view.

| Check | Applies to | Pass criteria |
|---|---|---|
| No source paths | All context files | Zero references to specific files or directories |
| Business language only | `domain.context.md` | Zero types, code snippets, or framework names |
| Patterns not listings | `implementation.context.md` | Zero file names or directory structure |
| No delegation | All context files | No "see X for details" phrases |
| Refactoring survival | All context files | Every line survives a behavior-preserving code refactor |
| Size limit | Per type | Root combined < 500 lines, domain < 300, implementation < 300 |
| No duplication | All context files | No information repeated from another context file |
| Markdown links | `project.context.md` | Documentation entries are `[Topic](docs/file.md)` links |
| Standalone | All context files | Each file is self-contained and understandable on its own |
| Writing discipline | All context files | Passes all line-level, language, and draft-level tests above |
| User approved | All context files | Explicit user approval received before writing to disk |

## Behavioral Rules

- **Never ask more than 3 questions at once.** Keep the conversation focused.
- **Never write a document without presenting it for review.** The user must see every draft.
- **Never proceed without explicit user approval.** Wait for confirmation.
- **Never auto-advance to the next phase or document.** Always ask before moving on.
- **Push back on anti-patterns.** If user input contains source paths, types in domain context, or file listings in implementation context, redirect toward the correct abstraction level.
- **State what you know before asking.** Don't ask questions the README already answered. Show your understanding and ask for corrections.

