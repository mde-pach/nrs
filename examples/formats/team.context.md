# Team Context — Commerce Team

## Scope

Owns product catalog, order management, and payment integration.

## Processes

- **Deployments**: Tuesdays and Thursdays, 10:00–12:00 UTC. Hotfixes anytime with team lead approval.
- **On-call**: Weekly rotation. Escalation: Slack `#commerce-oncall` then PagerDuty.
- **PR reviews**: <24h turnaround.

## Deviations from Corporate Standards

- Redis for session caching instead of corporate Memcached — approved exception for pub/sub requirements on real-time inventory
- Integration tests run against shared staging database — migration to isolated containers in progress
