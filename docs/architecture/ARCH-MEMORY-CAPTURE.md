# ARCH-MEMORY-CAPTURE: Memory Capture Architecture

**Version:** 1.0  
**Status:** FROZEN  
**Date:** 2026-06-22  
**ADR:** This document defines the architectural contract for memory acquisition in ARES MemoryOS. Changes require: ADR, Migration Plan, Re-Certification, Version Bump.

---

## 1. Core Principle

```
Capture Layer = Facts
Intelligence Layer = Interpretations

Never mix them.
```

The Memory Capture system records observable facts from repository sources. It does **not** infer requirements, decisions, or ownership from those facts. Inference is the responsibility of the Intelligence Layer (P3.3+).

---

## 2. Source Hierarchy

Memory sources are classified into three tiers based on reliability and availability.

### Tier 1: Explicit Sources (Confidence Base = 1.0)

Human-authored artifacts with clear intent.

| Source | Produces | Method |
|--------|----------|--------|
| `CODEOWNERS` | `Person` + `Owns` edge | File parse |
| `docs/requirements/` | `Requirement` node | Markdown parse |
| `docs/decisions/` | `Decision` node | Markdown parse |
| `docs/architecture/` | Architecture node | Markdown parse |
| ADR files (`ADR-xxx`) | `Decision` node | Markdown parse |

### Tier 2: Repository Sources (Confidence Base = 0.8)

Machine-readable facts embedded in the repository itself.

| Source | Produces | Method |
|--------|----------|--------|
| `git log` | `Commit` nodes | `git log --format` |
| `git tag` | `Release` nodes | `git tag -l` |
| `git branch` | `Branch` nodes | `git branch -a` |
| `git blame` | `ContributedTo` edges | `git blame --porcelain` |

### Tier 3: External Sources (Confidence Base = 0.6)

Sources that require network access or API tokens. **Deferred to P3.4.**

| Source | Produces | Method |
|--------|----------|--------|
| GitHub Issues | Requirement candidates | GitHub API |
| GitHub PRs | Decision candidates | GitHub API |
| GitHub Reviews | Approval evidence | GitHub API |
| Jira/Linear | Requirement candidates | API |

---

## 3. Confidence Hierarchy

Every captured fact carries a confidence score based on its provenance.

| Method | Confidence | Example |
|--------|-----------|---------|
| Explicit | 1.0 | CODEOWNERS entry, ADR document |
| Repository | 0.8 | Git tag, git commit metadata |
| Inferred | 0.6 | Conventional commit classification |
| Heuristic | 0.4 | Git blame line attribution |

### Confidence Rules

1. When multiple sources provide the same fact, the **highest confidence** wins.
2. CODEOWNERS always overrides git blame for ownership.
3. Confidence scores are stored, never discarded.
4. The governance engine uses confidence thresholds per policy.

---

## 4. Node Types (Capture Layer)

### New Node Types for P3.2

| NodeType | Description | Source |
|----------|-------------|--------|
| `Person` | A human contributor (author, owner, maintainer) | git log, CODEOWNERS, git blame |
| `Commit` | A single git commit (atomic fact) | git log |
| `Release` | A tagged release point | git tag |
| `Branch` | A git branch (engineering intent) | git branch |

### Existing Node Types (Unchanged)

`Project`, `File`, `Function`, `Method`, `Class`, `Struct`, `Enum`, `Trait`, `Interface`, `Module`, `Folder`, `Service`, `Decision`, `Feature`, `Bug`, `Concept`, `Tag`, `Requirement`, `Alternative`, `Assumption`, `Risk`

---

## 5. Edge Types (Capture Layer)

### New Edge Types for P3.2

| EdgeType | From → To | Description | Confidence |
|----------|-----------|-------------|------------|
| `ContributedTo` | Person → File | Author has contributed code to this file | 0.4–0.7 |
| `Maintains` | Person → File/Module | Person is the primary maintainer | 0.9 |
| `Touches` | Commit → File | Commit modified this file | 0.8 |
| `AuthoredBy` | Commit → Person | Commit was authored by this person | 1.0 |
| `ReleasedIn` | Commit → Release | Commit is included in this release | 0.8 |

### Ownership Edge Hierarchy

```
Owns           (from CODEOWNERS)       → confidence 1.0
Maintains      (frequent + recent)     → confidence 0.9
ContributedTo  (has authored commits)  → confidence 0.4–0.7
```

The governance engine computes **Ownership Coverage** using:

```
Owns OR Maintains
```

**Not** `ContributedTo`. Contribution is evidence, not authority.

### Existing Edge Types (Unchanged)

`Imports`, `Defines`, `Calls`, `Extends`, `DependsOn`, `Implements`, `Caused`, `FixedBy`, `Supersedes`, `MotivatedBy`, `Impacts`, `Owns`, `Authored`, `RelatedTo`, `TemporalFollows`, `Contradicts`, `Uses`, `DerivedFrom`, `Contains`, `ContainedIn`, `Invokes`, `Constructs`, `References`, `ResolvedTo`, `UsesModule`, `UsesTrait`, `Constrains`, `HasRisk`, `HasAssumption`, `Drives`, `Satisfies`, `OwnedBy`, `SupportedBy`, `ValidatedBy`

---

## 6. Source Provenance

Every captured node and edge carries provenance metadata in its `properties` field:

```json
{
  "source_system": "git_blame",
  "source_id": "abc123def456",
  "capture_method": "heuristic",
  "captured_at": 1750582400000,
  "confidence": 0.4
}
```

### CaptureMethod Enum

```rust
pub enum CaptureMethod {
    Explicit,    // CODEOWNERS, ADR, Requirements docs
    Repository,  // git log, git tag, git branch
    Inferred,    // Conventional commit classification
    Heuristic,   // git blame line attribution
}
```

---

## 7. Memory Source Registry

The Memory Source Registry tracks which memory sources exist in a repository and which have been captured.

```rust
pub struct MemorySourceRegistry {
    pub discovered_sources: Vec<MemorySource>,
    pub captured_sources: Vec<MemorySource>,
    pub unavailable_sources: Vec<MemorySource>,
}

pub struct MemorySource {
    pub name: String,          // "git_log", "codeowners", "github_issues"
    pub tier: SourceTier,      // Explicit, Repository, External
    pub available: bool,       // Does this repo have this source?
    pub captured: bool,        // Did ARES capture it?
    pub node_count: u64,       // How many nodes were produced
    pub edge_count: u64,       // How many edges were produced
}
```

### Memory Capture Rate

```
Memory Capture Rate = Captured Sources / Available Sources × 100
```

This is the **leading indicator** for memory quality.  
Coverage is the **lagging indicator**.

---

## 8. Capture Principles

### DO

1. Record git commits as `Commit` nodes with SHA, message, author, date
2. Record git tags as `Release` nodes with tag name and date
3. Record git branches as `Branch` nodes with branch name
4. Record CODEOWNERS entries as `Person` + `Owns` edges
5. Record git blame attribution as `Person` + `ContributedTo` edges
6. Store provenance metadata on every captured fact
7. Use confidence scores based on the hierarchy

### DO NOT

1. Do not infer ownership from git blame alone
2. Do not classify commits as requirements or decisions
3. Do not create "clusters" or "groups" of commits
4. Do not hallucinate edges that don't exist in the source data
5. Do not mix capture-layer logic with intelligence-layer logic

### Governance Integration

The coverage engine recognizes captured facts through their edge types:
- `Owns` / `Maintains` edges → count toward Ownership Coverage
- `Touches` edges → count toward Evolution tracking
- `ContributedTo` edges → stored but NOT used for Ownership Coverage

---

## 9. Graph Topology

### Release Traceability Chain

```
Release
  ↓ Contains
Commit
  ↓ Touches
File
```

### Branch Intent Chain

```
Branch
  ↓ Contains
Commit
  ↓ Touches
File
```

### Ownership Chain

```
Person
  ↓ Owns (CODEOWNERS, confidence 1.0)
File

Person
  ↓ Maintains (frequent + recent, confidence 0.9)
File

Person
  ↓ ContributedTo (blame, confidence 0.4–0.7)
File
```

### Commit Authorship

```
Commit
  ↓ AuthoredBy
Person
```

---

## 10. CLI Integration

```bash
# Full ingest with git memory capture
ares ingest .

# Control commit depth
ares ingest . --git-depth 500
ares ingest . --git-depth 5000
ares ingest . --git-depth all

# First ingest: full blame (cached)
# Subsequent: incremental blame (changed files only)
```

---

## 11. Version History

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-06-22 | Initial frozen definition |
