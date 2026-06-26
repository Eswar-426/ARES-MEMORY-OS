# ADR-006: Memory Bootstrap Intelligence

## Status
Accepted (P12)

## Context
ARES Memory OS assumes a top-down metadata architecture (`Requirement -> Decision -> Architecture -> Code`). However, real-world repositories (and ARES itself) are built "Code-First", lacking explicit traceability metadata. This creates a massive "Bootstrapping Gap" where ARES cannot inherently answer *why* code exists or *who* owns conceptual capabilities without manual user annotation. 

We need a deterministic system to bootstrap this missing memory from raw source code, dependency graphs, file structures, and commit histories.

## Decision
We will introduce `ares-memory-bootstrap`, a crate dedicated to Memory Bootstrap Intelligence. 

### 1. The Candidate Wrapper Model
Instead of polluting the core ontology with new node types like `InferredRequirement` or `InferredDecision`, all bootstrapped memory will be generated as generic `Candidate` nodes via the `ares-candidates` crate. 
- The payload of the Candidate contains the inferred structural memory.
- Candidates remain strictly isolated from authoritative Memory Graph queries until explicitly approved.
- Every Candidate must record its provenance: `commit_hash`, `repository_id`, `rule_id`, `engine_version`, and `generated_at`.

### 2. RuleProvider Abstraction
Rules for heuristics (e.g., inferring "Async Web Stack" from `tokio` and `axum`) will not be hardcoded as monolithic Rust logic. We will implement a `RuleProvider` trait with concrete `BuiltInRules` and `YamlRules` implementations to anticipate future plugin systems.

### 3. The Engine Suite
The bootstrapping will be performed by six deterministic engines:
1. `CapabilityInferenceEngine`
2. `ArchitectureInferenceEngine`
3. `DecisionInferenceEngine`
4. `OwnershipInferenceEngine`
5. `RequirementInferenceEngine`
6. `MemoryGapBootstrapEngine` (Connects directly to P7 to propose memory exclusively for detected gaps)

## Consequences
**Positive:**
- ARES becomes immediately useful on massive legacy repositories.
- `Candidate` isolation ensures 0% fact pollution in the authoritative graph.
- The `MemoryGapBootstrapEngine` creates a powerful flywheel: ARES finds missing memory, proposes candidates, and the human simply approves.

**Risks:**
- Candidate sprawl on massive monorepos (like Tokio) could overwhelm human reviewers. The system must implement confident filtering and bulk-approval workflows.
- Heuristics must remain deterministic; any LLM enrichment will be implemented strictly as a non-core additive layer in the future.
