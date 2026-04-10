# Corporate Context

## Observability

- **Error tracking**: Sentry — all unhandled exceptions must be captured, never silently caught
- **Logging**: OpenSearch — structured JSON logs, levels: `error` (pages oncall), `warn` (dashboard), `info` (audit), `debug` (local only)
- **Monitoring**: Datadog — every service exposes health and readiness endpoints

## Tools

- **Tickets**: Jira — every branch references a ticket
- **Communication**: Slack — `#engineering-alerts` for incidents, `#code-review` for PRs
- **CI/CD**: GitHub Actions — all PRs require passing CI before merge

## Standards

- Authentication via corporate OAuth2 SDK — never roll custom auth
- API contracts: OpenAPI 3.1 specs committed alongside implementation, breaking changes require RFC
- PII encrypted at rest, never logged, GDPR deletion handled via corporate toolkit

## Code Practices

- Formatter and linter run on precommit — no manual formatting
- All PRs require at least one review
- Main branch is always deployable
