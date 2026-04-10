# Implementation Context — Product Services

## Patterns

- Service functions are stateless — no class instances, dependencies passed as arguments
- Services throw typed domain errors — they never return HTTP concepts, the API layer maps errors to status codes
- Multi-entity mutations use database transactions via the repository layer
