# Implementation Context — Product Tests

## Patterns

- Integration tests against a real SQLite database — no mocks for persistence
- Each test gets a fresh database state via full table truncation
- Tests verify both the return value and the database state after mutation
- Domain errors are tested by asserting the specific error type, not the message
- See Testing documentation for test database setup and test shape conventions
