# Project Context — Hello World Commerce

## Purpose

Minimal e-commerce application demonstrating NRS context organization with both frontend and backend concerns.

## Architecture

- Next.js (App Router) — frontend and API routes
- SQLite via Prisma — persistence
- Tailwind CSS — styling
- Vitest — integration testing against real database

## Domains

- **Products** — catalog and inventory management
- **Orders** — cart and checkout

Domains communicate through service interfaces, never direct cross-domain database access.

## Key Decisions

- Server Components by default, Client Components only for interactivity
- Monetary values stored as integers (cents)

## Documentation

- [Testing](docs/testing.md) — test framework, database setup, test shapes and conventions
- [Server Components](docs/server-components.md) — rendering strategy, client/server wrapper pattern

## Commands

- `npm run dev` — start development server
- `npm run build` — production build
- `npm test` — run integration tests
- `npm run db:setup` — push schema and seed database
