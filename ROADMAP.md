# Roadmap

## Planned

### Spec & Guidelines

### Tooling

- **Gap resolution skill** — A dedicated skill (like nrs-setup) that reads `nrs.gaps.md`, picks gaps prioritized by frequency, proposes context file updates to the user, and removes resolved rows in the same commit. The consumption side of gap reporting — separate from the development workflow where gaps are only logged.


## Done

- Context layer system (layers 1–7)
- CLI: generate, validate, init, install
- CLI: gap report + gap summary
- Validators: size, source paths, references, links, duplication, orphan docs, generated drift
- Claude generator + settings.local.json ignore patterns
- nrs-setup skill for Claude Code
- Pre-commit hook automation
- Sub-agent guideline: gathering vs implementation ([Cemri et al., 2025](https://arxiv.org/abs/2503.13657), [Cognition, 2025](https://cognition.ai/blog/dont-build-multi-agents), [Chen et al., 2026](https://arxiv.org/abs/2603.22608))
- Output discipline agent guideline ([Lindenbauer et al., NeurIPS DL4Code 2025](https://arxiv.org/abs/2508.21433))
- Required `## Commands` section in project.context.md — validator enforces presence ([Haller et al., 2026](https://arxiv.org/abs/2602.11988))
- Test-double guidance in docs/ — when to mock vs real, type selection table ([Hora, MSR 2026](https://arxiv.org/abs/2602.00409))
- Prescribe docs/ folder hierarchy — folder structure reflects abstractions ([Cao et al., 2026](https://arxiv.org/abs/2603.20432))
