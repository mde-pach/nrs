# NRS

An opinionated framework for agentic context and codebase organization. NRS structures how context is layered, referenced, and maintained to optimize both human and AI-agent workflows.

Built on Domain-Driven Design. Backed by [academic research](SPEC.md#references).

## Why

AI coding agents perform worse — not better — without structured context. Unstructured AI usage increased experienced developer completion time by 19%[9], while structured repository documentation reduced agent runtime by 28.6%[5]. Context length alone degrades LLM performance by 13.9–85%[2], and irrelevant context actively harms reasoning[3].

NRS solves this by defining:
- **What context exists** — a 7-layer system from developer preferences to inline code
- **Where it lives** — co-located with the code it describes, never centralized
- **How it's referenced** — strict rules preventing invisible coupling
- **How much is allowed** — research-backed size limits and density requirements
- **How it reaches tools** — generated entry points for Claude Code, Cursor, Gemini, etc.

## Context Layers

```
developer context              ← personal, not committed
  corporate context            ← company-wide (corporate.context.md)
    team context               ← team conventions (team.context.md)
      project context          ← purpose & architecture (project.context.md)
        domain context         ← business concepts (domain.context.md)
          implementation context  ← patterns & decisions (implementation.context.md)
            code & inline docs    ← the source files
```

Each inner layer is contained within the outer. Reference rules: same level is allowed, never reference higher (outer), should not reference lower (inner).

## File Structure

Context files are co-located with the code they describe. The layer is encoded in the filename.

```
project/
├── CLAUDE.md                    ← generated, DO NOT EDIT
├── .cursorrules                 ← generated, DO NOT EDIT
├── corporate.context.md         ← Layer 2
├── team.context.md              ← Layer 3
├── project.context.md                   ← Layer 4
└── src/
    ├── billing/
    │   ├── domain.context.md    ← Layer 5
    │   └── services/
    │       ├── implementation.context.md  ← Layer 6
    │       └── ...
    └── ...
```

Tool entry points (CLAUDE.md, .cursorrules, etc.) are generated from layers 2–4 and regenerated on every commit. Domain and implementation contexts are discovered on-demand by agents during work.

## Quick Start

See `examples/hello-world/` for a complete Next.js project with NRS context files at every layer.

See `examples/formats/` for standalone examples of each context file type.

## Specification

The full spec with research references is in [SPEC.md](SPEC.md).

___

[2]: https://arxiv.org/abs/2510.05381
[3]: https://arxiv.org/abs/2302.00093
[5]: https://arxiv.org/abs/2601.20404
[9]: https://arxiv.org/abs/2507.09089
