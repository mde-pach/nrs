# Testing

## Framework

Vitest with integration tests against a real SQLite database. No mocks for persistence — tests hit the actual database to catch issues that mocks would hide.

## Test Database Setup

Tests use a dedicated test database separate from the development database. The setup:

1. A shared helper provisions the test database by pushing the Prisma schema to a test-specific SQLite file
2. Before each test, all tables are truncated in dependency order (child tables first) to ensure a clean state
3. After the suite completes, the test database file is deleted

The database is created once per test process and shared across test files. Tests run sequentially (single fork) to avoid file lock conflicts on SQLite.

## Test Shape

Each test follows the same structure:

1. **Arrange**: Create the required database records (categories, products, carts) directly via Prisma
2. **Act**: Call the service function under test
3. **Assert**: Verify both the return value AND the database state after the operation

For error cases:
1. **Arrange**: Set up the conditions that should trigger the error
2. **Act + Assert**: Verify the specific error type is thrown (not the message — messages can change)
3. **Assert side effects**: Verify no partial state was written (e.g., stock unchanged after a failed checkout)

## What to Test

- Service functions, not repositories — services contain the business logic, repositories are data access
- Success paths with full state verification (return value + database)
- Every typed domain error the service can throw
- Transaction rollback: when a multi-step operation fails, verify nothing was partially committed
- Cross-domain side effects: when an operation touches another domain's data, verify that too
