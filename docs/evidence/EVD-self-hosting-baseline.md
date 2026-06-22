# Evidence: Self-Hosting Baseline
## ID: EVD-self-hosting-baseline

This evidence supports ADR-005.

The self-hosting script generated metrics natively from the ARES SQLite DB without requiring any cloud APIs or external parsers. This validates the local-first architecture and proves ARES can introspect its own health securely and offline.

Detailed metrics are available in `reports/validation/self_hosting_readiness.md`.
