# Implementation Context — Order Tests

## Patterns

- Integration tests against a real SQLite database — no mocks for persistence
- Each test gets a fresh database state via full table truncation
- Domain errors are tested by asserting the specific error type, not the message
- Cross-domain side effects are verified (inventory changes after checkout)
- Transaction rollback is verified by checking that failed operations leave no partial state
- See Testing documentation for test database setup and test shape conventions
