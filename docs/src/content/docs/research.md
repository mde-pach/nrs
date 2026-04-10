---
title: Research
description: Academic papers backing NRS design decisions.
---

## Context & Attention

| Finding | Supports | Source |
|---|---|---|
| LLM performance drops when key info is in the middle of input | Placement rules | [Liu et al., TACL 2024](https://arxiv.org/abs/2307.03172) |
| Longer context degrades performance even with perfect retrieval | Size limits | [Du et al., EMNLP 2025](https://arxiv.org/abs/2510.05381) |
| Irrelevant context significantly harms reasoning | Density rules | [Shi et al., ICML 2023](https://arxiv.org/abs/2302.00093) |
| Removing noise through compression improves accuracy | On-demand loading | [Jiang et al., ACL 2024](https://arxiv.org/abs/2310.06839) |
| Verbose content underperforms concise across all models | Writing constraints | [Verbosity != Veracity, 2024](https://arxiv.org/abs/2411.07858) |

## Structured Documentation

| Finding | Supports | Source |
|---|---|---|
| Structured docs reduce agent runtime and token usage | Layer system | [Lulla et al., 2026](https://arxiv.org/abs/2601.20404) |
| Hierarchical doc architecture validated across hundreds of sessions | Concentric layers | [Vasilopoulos, 2026](https://arxiv.org/abs/2602.20478) |
| Ad-hoc context files have significant content duplication | Structured layers | [Jiang & Nam, MSR 2026](https://arxiv.org/abs/2512.18925) |
| Human-written context outperforms LLM-generated | Size limits | [Evaluating AGENTS.md, 2026](https://arxiv.org/abs/2602.11988) |

## Agent Architecture

| Finding | Supports | Source |
|---|---|---|
| Multi-agent with focused roles outperforms monolithic | Sub-agent strategy | [Huang et al., 2024](https://arxiv.org/abs/2312.13010) |
| AI tools increase completion time without structure | Why NRS exists | [Becker et al., 2025](https://arxiv.org/abs/2507.09089) |
| Codebase structure is the critical barrier for agents | Navigability | [Jimenez et al., ICLR 2024](https://arxiv.org/abs/2310.06770) |
| LLM coupling reasoning collapses in noisy scenarios | Reference rules | [Saad et al., 2025](https://arxiv.org/abs/2511.20933) |
| Instance count degrades performance independently of length | Sub-agent decomposition | [Chen et al., 2026](https://arxiv.org/abs/2603.22608) |

## Autonomy & Testing

| Finding | Supports | Source |
|---|---|---|
| Agent autonomy must be a deliberate design choice | Propose first | [Feng et al., 2025](https://arxiv.org/abs/2506.12469) |
| Majority of developers encounter flaky tests regularly | Deterministic tests | [Parry et al., TOSEM 2022](https://dl.acm.org/doi/10.1145/3476105) |
| TDD significantly reduces defect density in industrial settings | Testing philosophy | [Nagappan et al., 2008](https://link.springer.com/article/10.1007/s10664-008-9062-z) |
