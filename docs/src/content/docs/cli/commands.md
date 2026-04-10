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

| Target | Output | Ignore config |
|---|---|---|
| `claude` | `CLAUDE.md` | `.claude/settings.local.json` |

Ordering: nrs → corporate → team → project → domain → implementation.

---

## `nrs validate`

Checks all context files. Exit code 1 on errors.

```bash
nrs validate
nrs validate --dir /path/to/project
```

| Check | Severity |
|---|---|
| Size limits exceeded | warning |
| Source file paths in context | error |
| Generated files out of date | error |
| Broken links in project.context.md | error |
| Reference rule violations | error |

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
