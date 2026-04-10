---
name: nrs-setup
description: Set up NRS (Nested Reference System) in an existing project. Use when asked to "set up NRS", "initialize NRS", "add NRS to this project", "create context files", or when working with a project that needs agentic context organization. Handles creating nrs.context.md, project.context.md, domain contexts, implementation contexts, docs/, and running the NRS CLI for generation and validation.
---

# NRS Setup

Set up NRS context organization in an existing project through an interactive workflow.

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

## Workflow

### Phase 0 — Understand the Project

Read the project's README (and any existing documentation) before doing anything. This gives the baseline understanding of what the project does, its tech stack, and how it's structured. Use this to inform all subsequent phases — don't ask the user questions that the README already answers.

### Phase 1 — Initialize

```bash
nrs init
```

Creates `nrs.context.md` (agent operating rules) and the precommit hook.

### Phase 2 — Project Map

Using what was learned from the README, draft `project.context.md` and confirm with the user. Ask only what the README doesn't cover:

1. What does the project do?
2. Tech stack (framework, database, key tools)
3. What are the main areas/domains of the codebase?
4. What commands are available (dev, build, test, deploy)?
5. Any documentation topics worth writing up?

Write `project.context.md` following format in [references/context-formats.md](references/context-formats.md). Documentation entries must be markdown links to `docs/` files. Create `docs/` directory and referenced files if the user identifies topics.

### Phase 3 — Corporate and Team (optional)

Ask if company-wide standards or team conventions exist. Skip if not.

### Phase 4 — Domain Contexts

Explore the codebase to identify domain boundaries — look for directories representing business areas with distinct vocabulary. For each, ask the user to describe it in business terms. Write `domain.context.md` in the domain directory.

Rules: business language only, no types or code, each file standalone.

### Phase 5 — Implementation Contexts

Only where specific patterns need documenting. Ask: "Is there something specific about how this area is built that a developer needs to know?" Do NOT create one everywhere.

Rules: patterns and decisions only, no file listings.

### Phase 6 — Generate and Validate

```bash
nrs generate claude
nrs validate
```

Fix validation errors: remove source paths, rewrite implementation terms in domain contexts as business language, create missing linked docs.

## Anti-Patterns to Prevent

- Source file paths in any context file
- Type annotations or code in domain context
- File listings in implementation context
- Delegating understanding to another context ("see X for details") — stating facts about related domains is fine
- Creating implementation.context.md with nothing specific to say
- Documentation links as plain text instead of markdown links
