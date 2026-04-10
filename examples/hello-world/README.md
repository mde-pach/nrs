# Hello World Commerce

Minimal e-commerce application demonstrating NRS context organization in a Next.js project.

## Setup

```bash
npm install
npm run db:setup
npm run dev
```

## What to look at

This project uses NRS context files at every layer:

- `nrs.context.md` — NRS operating rules for agents
- `corporate.context.md` — company-wide standards
- `team.context.md` — team conventions
- `project.context.md` — project purpose and architecture
- `src/domains/*/domain.context.md` — business concepts and rules
- `src/domains/*/implementation.context.md` — domain-level implementation patterns
- `src/app/api/implementation.context.md` — API-level patterns
- `src/app/api/orders/implementation.context.md` — route-level exception context

## Domains

- **Products** — catalog and inventory
- **Orders** — cart and checkout
