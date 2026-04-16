# Docs Site — Agent Guide

Astro + Starlight site that presents NRS to the public. This is a **presentation layer**, not a source of truth. The spec lives in `../SPEC.md`.

## Package Manager

**Bun only.** `bun.lock` is the tracked lockfile; `package-lock.json` is gitignored. Never run `npm install` here — it will create a drift lockfile that the gitignore hides. If bun is missing, install it before working in this directory.

## Commands

Run from this directory:

- `bun install` — install dependencies
- `bun run dev` — local dev server (Astro)
- `bun run build` — production build into `dist/`
- `bun run preview` — serve the built site

## Layout

- `src/content/docs/` — all user-facing pages (`.md` / `.mdx`). Top level today: `index.mdx`, `overview.mdx`, `quickstart.mdx`, `example.mdx`, `research.md`, plus `cli/` and `concepts/` subfolders.
- `src/components/` — Astro components used by pages.
- `src/assets/` — images and static assets imported by components.
- `src/contexts/` — page-level contexts (Astro primitives, not NRS context files).
- `content.config.ts` — Starlight content collection config.
- `astro.config.mjs` — site config, integrations, base path.

## Writing Rules

- **Don't duplicate `SPEC.md`.** If a page needs to state a spec rule verbatim, link to the section in `SPEC.md` rather than copy-pasting. The docs site is allowed to paraphrase, illustrate, and teach — not to fork the spec.
- **No `*.context.md` files here.** This directory is outside the NRS layer system. The `docs/` directory that NRS describes in `SPEC.md` is a *per-project* `docs/` folder, not this Astro site.
- **Examples stay minimal.** Code snippets that illustrate a concept are fine. Full sample projects belong under `../examples/`.
- **Mermaid diagrams** are supported via `astro-mermaid`. Prefer them for layer diagrams over ASCII art.

## When Spec Changes

If `../SPEC.md` changes a rule or adds a concept, check whether any page under `src/content/docs/` (especially `overview.mdx`, `concepts/`, `research.md`) now says something false or stale, and update it in the same PR.
