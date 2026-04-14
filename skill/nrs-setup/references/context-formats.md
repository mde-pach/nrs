# NRS Context File Formats

## Naming Convention

| File | Layer | Location |
|---|---|---|
| `nrs.context.md` | Baseline agent behavior | Project root |
| `corporate.context.md` | Corporate | Project root |
| `team.context.md` | Team | Project root |
| `project.context.md` | Project map | Project root |
| `domain.context.md` | Domain | Domain directory |
| `implementation.context.md` | Implementation | Implementation area |

All files match `*.context.md`. No other naming is valid.

## Rules

- No source file paths — never reference specific files or directories
- Domain context = business language — no types, no code, no framework names
- Implementation context = patterns — not file listings
- Same-level contexts may reference each other to state facts, but must not delegate understanding ("see X for details")
- Refactoring test: if code refactored without behavior change, would this line break? If yes, it belongs in code

## project.context.md Format

Acts as the project map. Must contain markdown links to docs.

```markdown
# Project Context — {Name}

## Purpose

{What the project does and why}

## Architecture

- {Framework} — {role}
- {Database} — {role}

## Domains

- **{Domain}** — {one-line description}

## Documentation

- [{Topic}](docs/{file}.md) — {description}

## Commands

- `{command}` — {description}
```

## domain.context.md Format

Business language only. Written by product.

```markdown
# Domain Context — {Name}

## Business Purpose

{What this domain does in business terms}

## Core Concepts

- **{Concept}**: {Business description without types or code}

## Business Rules

- {Rule in plain language}

## Domain Relations

- **{Other Domain}**: {How they relate in business terms}
```

## implementation.context.md Format

Patterns and decisions. Only create when there is something specific to say.

```markdown
# Implementation Context — {Area}

## Patterns

- {Pattern description — no file names, no directory listings}
```

## corporate.context.md Format

```markdown
# Corporate Context

## {Category}

- {Standard or guideline}
```

## team.context.md Format

```markdown
# Team Context — {Name}

## Scope

{What the team owns}

## Deviations from Corporate Standards

- {Deviation} — {reason}
```

## Size Limits

- Root contexts combined (nrs + corporate + team + project): ~500 lines max
- domain.context.md: ~300 lines max
- implementation.context.md: ~300 lines max
- Shorter is always better
