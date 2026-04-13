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

---

## `nrs gap summary`

Reads `nrs.gaps.md` and displays gaps grouped by target.

```bash
nrs gap summary
nrs gap summary --dir /path/to/project
```
