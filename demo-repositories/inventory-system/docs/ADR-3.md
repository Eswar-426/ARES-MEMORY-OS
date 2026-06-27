# ADR-3: Mandatory Database Access Pattern

**Status:** Accepted
**Date:** 2025-01-15
**Owner:** Platform Team

## Context
Direct database queries scattered across handlers lead to inconsistent caching, poor transaction boundaries, and untestable code.

## Decision
All database access must go exclusively through the `repository` layer (e.g., `src/repository/`). Handlers and business logic are strictly forbidden from connecting directly to the database or executing SQL statements.

## Consequences
- Better maintainability.
- All handlers must take `Repository` interfaces as dependencies.
