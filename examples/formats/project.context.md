# Project Context — Commerce Platform

## Purpose

Web application for managing a product catalog and processing customer orders. Serves both the customer-facing storefront and the internal back-office.

## Architecture

- Next.js (App Router) — serves both frontend and API routes
- PostgreSQL via Prisma ORM
- Redis for sessions and real-time inventory
- BullMQ for async order processing

## Domains

- **Products** — catalog management, search, inventory tracking
- **Orders** — cart, checkout, payment, fulfillment

Domains communicate through well-defined service interfaces, never direct database access across boundaries.

## External Dependencies

- Stripe for payment processing
- SendGrid for transactional emails
- Corporate OAuth2 SDK for authentication

## Key Decisions

- Server Components by default, Client Components only when interactivity is required
- API routes handle business logic, components handle presentation
- All monetary values stored as integers (cents) to avoid floating point issues

## Documentation

- [Testing](docs/testing.md) — test framework, database setup, test shapes and conventions
- [Server Components](docs/server-components.md) — rendering strategy, client/server wrapper pattern
- [Deployment](docs/deployment.md) — infrastructure, environments, rollback procedures
- [API Contracts](docs/api-contracts.md) — OpenAPI spec conventions, versioning

## Tools

- `npm run dev` — start development server
- `npm run build` — production build
- `npm test` — run integration tests
- `npm run db:setup` — push schema and seed database

## Skills

- `/deploy` — trigger deployment pipeline
- `/migrate` — run database migrations

## MCP Servers

- **Sentry** — error tracking queries
- **Jira** — ticket management
