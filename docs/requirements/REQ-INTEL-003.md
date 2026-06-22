# Requirement: Test Resolution
## ID: REQ-INTEL-003

The system must deterministically resolve test files for code artifacts using naming and path heuristics.

Test resolution must support:
- Rust: `src/payment.rs` → `tests/payment_test.rs` or `src/payment_test.rs`.
- TypeScript: `user.ts` → `user.spec.ts` or `user.test.ts`.
- JavaScript: `auth.js` → `auth.test.js` or `auth.spec.js`.
- Python: `auth.py` → `test_auth.py` or `auth_test.py`.

Each resolved pair produces a ValidatedBy edge from CodeArtifact to TestArtifact.

Parent: REQ-MEMORY-002.

This requirement is implemented in crates/ares-ingestion/src/extractors/tests.rs.
