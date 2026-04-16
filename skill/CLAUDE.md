# Skills — Agent Guide

Claude Code skills shipped with NRS. Installed into a user's Claude Code by `nrs install claude`.

## Skills

- **`nrs-setup/`** — interactive, progressive setup of NRS in an existing project. Creates context files one at a time with the user, per the single-document lifecycle (gather → draft → review → confirm). Invoked by phrases like "set up NRS", "initialize NRS".
- **`nrs-fix/`** — consumes `nrs.gaps.md`, proposes updates to context files, removes resolved rows in the same commit. Signal-aware: prioritizes gaps by frequency and confidence.

## File Conventions

- Each skill is a directory containing `SKILL.md` (and optionally `references/`).
- `SKILL.md` starts with YAML frontmatter: `name`, `description`. The description is the routing contract — Claude Code reads it to decide when to invoke the skill. Be explicit about trigger phrases.
- Skill directories may include supporting files (templates, reference docs) referenced by relative path from `SKILL.md`.
- `skill/*.skill` (packaged artifacts) are gitignored — only the source `SKILL.md` + assets are tracked.

## Relation to the CLI

Skills **drive** the CLI, they do not replace it. When `nrs-setup` creates a project, it does so by running `nrs init`, `nrs generate claude`, `nrs validate`. When the CLI changes its subcommand interface, the affected skill must be updated in the same commit — otherwise users get a skill that calls flags that no longer exist.

## Writing Rules

- **Describe behavior, not internals.** The skill's description should match what a user asks for, not what the skill does under the hood.
- **Explicit lifecycles.** If the skill has stages (gather, draft, review, confirm), spell them out as numbered steps. Claude follows the structure it reads.
- **Reference, don't inline.** When the skill needs long reference material, put it in `references/` inside the skill and link from `SKILL.md` — don't bloat `SKILL.md` itself.
- **No tool-specific hacks.** Skills should behave correctly whether invoked via `/skill-name`, a phrase match, or an agent delegation.
