# References — Agent Guide

Cited academic papers backing `../SPEC.md`. One PDF per numbered citation.

## Naming Convention

`[N]-firstauthor-year-slug.pdf`

- `N` — the citation index used in `SPEC.md` (e.g. `[6]`). Indexes are append-only; never renumber existing entries.
- `firstauthor` — surname of the first author, lowercase.
- `year` — 4-digit publication year.
- `slug` — short hyphenated phrase capturing the paper's contribution.

Example: `[23]-haller-2026-evaluating-agents-md.pdf`.

## Rules

- **Every PDF here must be cited by `[N]` in `SPEC.md`.** An uncited PDF is dead weight — delete it or add the citation.
- **Every `[N]` in `SPEC.md` must have a PDF here.** Broken citations are a validator-level integrity issue; don't let them slip in.
- **Never renumber.** If a paper becomes obsolete and you remove it, leave its index retired — don't reuse the number. This keeps historical PRs and git history readable.
- **One claim per citation.** When adding a paper, quote the specific finding in the `SPEC.md` sentence (e.g. "degrades 13.9%–85%"). A citation without a concrete claim in the prose is filler.
- **Keep the `[N]: URL (author. "Title." venue, year.)` block at the bottom of `SPEC.md` sorted by index.**

## Adding a Reference

1. Drop the PDF in this directory with the correct filename.
2. Add the `[N]` inline citation at the point in `SPEC.md` prose where it supports a concrete claim.
3. Add the `[N]: URL (citation text)` entry in the reference block at the bottom of `SPEC.md`.
4. Commit all three changes together.
