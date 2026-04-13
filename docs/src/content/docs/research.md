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
| Semantic density matters more than token minimization — aggressive compression increased costs by 67% | Density rules | [Ustynov, 2026](https://arxiv.org/abs/2604.07502) |
| Observation masking matches LLM summarization at 52% lower cost; summarization causes trajectory elongation | Output discipline | [Lindenbauer et al., NeurIPS 2025](https://arxiv.org/abs/2508.21433) |

## Structured Documentation

| Finding | Supports | Source |
|---|---|---|
| Structured docs reduce agent runtime and token usage | Layer system | [Lulla et al., 2026](https://arxiv.org/abs/2601.20404) |
| Hierarchical doc architecture validated across hundreds of sessions | Concentric layers | [Vasilopoulos, 2026](https://arxiv.org/abs/2602.20478) |
| Ad-hoc context files have significant content duplication | Structured layers | [Jiang & Nam, MSR 2026](https://arxiv.org/abs/2512.18925) |
| No established standard for context file structure across 466 OSS projects | Why NRS exists | [Mohsenimofidi et al., 2025](https://arxiv.org/abs/2510.21413) |
| Context files neglect security (14.5%) and performance (14.5%) without structure | Deliberate layers | [Chatlatanagulchai et al., 2025](https://arxiv.org/abs/2511.12884) |
| LLM-generated context files decrease success by 2–3% and increase cost 20%; developer-written improve by ~4%. Tooling commands are highest-ROI content. | Density rules, tooling guidance | [Haller et al., 2026](https://arxiv.org/abs/2602.11988) |
| Context collapses through iterative rewriting without structured incremental updates | Layer maintenance | [Zhang et al., 2025](https://arxiv.org/abs/2510.04618) |

## Agent Architecture

| Finding | Supports | Source |
|---|---|---|
| Multi-agent with focused roles outperforms monolithic | Sub-agent strategy | [Huang et al., 2024](https://arxiv.org/abs/2312.13010) |
| AI tools increase experienced developer completion time by 19% | Why NRS exists | [Becker et al., 2025](https://arxiv.org/abs/2507.09089) |
| Codebase structure is the critical barrier for agents | Navigability | [Jimenez et al., ICLR 2024](https://arxiv.org/abs/2310.06770) |
| LLM coupling reasoning collapses in noisy scenarios | Reference rules | [Saad et al., 2025](https://arxiv.org/abs/2511.20933) |
| Instance count degrades performance independently of length | Sub-agent decomposition | [Chen et al., 2026](https://arxiv.org/abs/2603.22608) |
| Coding agents with file system tools outperform long-context approaches by 17.3% | Sub-agent strategy | [Cao et al., 2026](https://arxiv.org/abs/2603.20432) |
| 75.3% of multi-agent failures stem from planner-to-coder information degradation | Chained workflow | [Lyu et al., 2025](https://arxiv.org/abs/2510.10460) |
| AI tool adoption increases code complexity by 40.7% and static warnings by 29.7% (persistent); velocity gains vanish by month 3 | Testing philosophy | [He et al., MSR 2026](https://arxiv.org/abs/2511.04427) |
| Agents over-mock tests (36% vs 26% human rate), 95% mock type usage vs diverse human strategies | Testing guidance | [Hora, MSR 2026](https://arxiv.org/abs/2602.00409) |
| 14 failure modes in multi-agent systems: system design, inter-agent misalignment, task verification | Sub-agent guidelines | [Cemri et al., 2025](https://arxiv.org/abs/2503.13657) |

## Autonomy & Testing

| Finding | Supports | Source |
|---|---|---|
| Agent autonomy must be a deliberate design choice | Propose first | [Feng et al., 2025](https://arxiv.org/abs/2506.12469) |
| Majority of developers encounter flaky tests regularly | Deterministic tests | [Parry et al., TOSEM 2022](https://dl.acm.org/doi/10.1145/3476105) |
| TDD significantly reduces defect density in industrial settings | Testing philosophy | [Nagappan et al., 2008](https://link.springer.com/article/10.1007/s10664-008-9062-z) |
