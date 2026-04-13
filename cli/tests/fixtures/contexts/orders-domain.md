# Orders Domain

Every order captures a snapshot of product prices at checkout time.
Orders transition through: draft → confirmed → fulfilled → closed.
Cancelled orders are soft-deleted and excluded from revenue reports.
