# Repository Boundary Audit

**Date**: 2026-06-22
**Component**: ARES Candidate Governance Pipeline

## Overview
This audit verifies that the ARES Memory OS strictly enforces repository boundaries across the entire lifecycle of requirement and decision candidates. No candidate data can leak across projects, and promotions are explicitly blocked if source and target differ in repository ownership.

## Validation Checklist

### 1. Schema Validation
- [x] **Every candidate contains a `project_id`**: The `Candidate` model and the `candidates` SQLite table both feature a mandatory `project_id` column.

### 2. Query Validation
- [x] **Every query requires a `project_id`**: The `CandidateRepository` trait has been refactored. The following methods now strictly require `project_id` to be passed alongside `candidate_id`:
  - `get_candidate(project_id, id)`
  - `get_sources(project_id, candidate_id)`
  - `get_reviews(project_id, candidate_id)`
  - `get_promotion(project_id, candidate_id)`
- [x] **No cross-repository leakage**: A query attempting to fetch a valid `candidate_id` but passing the wrong `project_id` will return `None`.

### 3. Promotion Validation
- [x] **Repository parity check**: The `promote_candidate` transaction explicitly enforces `candidate.project_id == node.project_id.as_str()`.
- [x] **Rejection of mismatched promotions**: An automated test (`test_candidate_acceptance_and_isolation`) demonstrates that attempting to promote `repo-a`'s candidate to a node belonging to `repo-b` results in: `Repository mismatch: Candidate and Node must belong to the same repository.`

## Conclusion
The candidate governance pipeline provides airtight isolation. ARES is structurally guaranteed to protect repository candidate spaces from cross-contamination. The audit **PASSES**.
