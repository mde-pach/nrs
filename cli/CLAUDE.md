# CLI — Agent Guide

Rust CLI that implements NRS: reads context files, runs validators, emits tool-specific artifacts, ingests Claude Code hooks. Binary name: `nrs`.

## Layout

- `src/main.rs` — clap entry point; one match arm per subcommand.
- `src/commands/` — one file per subcommand (`generate`, `validate`, `init`, `install`, `gap`, `observe`, `guard`, `layers`, `notify`). Thin wrappers; real work lives in the modules below.
- `src/model/` — typed representations of context files, layers, and the context set.
- `src/discovery.rs` — finds `*.context.md` files under a project root.
- `src/markdown.rs` — markdown parsing helpers.
- `src/validate/` — one file per validator: `size`, `source_paths`, `references`, `links`, `duplication`, `orphan_docs`, `generated_drift`, `required_sections`. Each exports a check function; all are registered in `mod.rs`.
- `src/generators/` — tool-specific emitters (today: `claude.rs`). Each generator consumes a `ContextSet` and writes files.
- `src/observe.rs` + `src/gaps.rs` — transcript signal detection, candidates staging (`nrs.gaps.candidates.md`), and confirmed gap management (`nrs.gaps.md`).
- `templates/` — static text pulled in at build or runtime (e.g. `claude-header.txt`, `nrs.context.md`).
- `tests/` — integration tests; one file per area (`generate`, `validate`, `init`, `gap`, `observe`, `notify`, `layers`, `e2e`). Fixtures under `tests/fixtures/`.

## Commands

From this directory (`cli/`) or with `--manifest-path cli/Cargo.toml` from repo root:

- `cargo build` — compile the CLI
- `cargo test` — run unit + integration tests
- `cargo run -- <subcommand>` — invoke without installing
- `cargo install --path .` — install the `nrs` binary locally

## Working Rules

- **Validators and generators are symmetric.** Any new rule in `validate/` that constrains what layer context files may contain also has consequences in `generators/` (what gets emitted) and in the spec (`SPEC.md`). Change all three in the same commit.
- **Every new subcommand gets an integration test** in `tests/`. Use `assert_cmd` + `tempfile` + `predicates` — already dev-dependencies. Follow the shape of an existing test file.
- **Hook mode is a protocol, not a flag.** Claude subcommands take `--hook-mode` to read JSON from stdin instead of CLI args. When adding a new Claude hook integration, keep the non-hook-mode path usable from a human terminal.
- **No panics in command paths.** Everything returns `anyhow::Result`. Let errors propagate to `main::run`.
- **Templates live in `templates/`**, not inlined as strings. Include via `include_str!` at the call site.

## Testing Discipline

Mirror what NRS preaches: evidence-based bug fixes. Write the failing integration test first, confirm it fails for the right reason, then fix. Flaky tests are unacceptable — if a detector fires intermittently in tests, raise input volume until deterministic.
