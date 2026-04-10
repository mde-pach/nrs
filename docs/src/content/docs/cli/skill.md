---
title: Skill
description: The NRS setup skill for agentic tools.
---

## What It Does

The `nrs-setup` skill guides an AI agent through bootstrapping NRS in an existing project. It reads the README, asks targeted questions, and creates context files interactively.

## Install

```bash
nrs install claude
```

## Trigger

Say any of:
- "set up NRS"
- "initialize NRS"
- "add NRS to this project"
- "create context files"

## Workflow

1. **Read README** — understand the project before asking questions
2. **`nrs init`** — creates `nrs.context.md` + hook
3. **Draft `project.context.md`** — from README + user input
4. **Corporate/team** — only if the user has content
5. **Domain contexts** — explore codebase, ask user to describe in business terms
6. **Implementation contexts** — only where specific patterns exist
7. **`nrs generate` + `nrs validate`** — generate and verify
