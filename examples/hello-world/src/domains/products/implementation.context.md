# Implementation Context — Products

## Patterns

- Service functions are stateless — dependencies passed as arguments
- Typed domain errors thrown on failure — services never return HTTP concepts, the API layer maps errors to status codes
- Repositories return domain types, never raw ORM types — mapping happens at the repository layer
- List queries are paginated by default (cursor-based)
- Multi-entity mutations use database transactions
