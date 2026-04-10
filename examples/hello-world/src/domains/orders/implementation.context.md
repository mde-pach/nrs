# Implementation Context — Orders

## Patterns

- Service functions are stateless — dependencies passed as arguments
- Typed domain errors thrown on failure — services never return HTTP concepts
- Repositories return domain types, never raw ORM types
- Checkout orchestrates across domains (inventory check + order creation) in a single transaction
- Cross-domain calls go through service functions, never direct repository or database access
