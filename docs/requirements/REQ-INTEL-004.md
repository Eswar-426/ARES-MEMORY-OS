# Requirement: Mutation Simulation
## ID: REQ-INTEL-004

The system must simulate structural changes to the knowledge graph and project their impact before they happen.

Simulation must support:
- Removing a node and projecting coverage drops, new gaps, and drift.
- Evaluating the effect of proposed changes on governance compliance.
- Returning a structured SimulationReport with projected metrics.

Parent: REQ-MEMORY-013.

This requirement is implemented in crates/ares-requirements/src/simulation.rs and crates/ares-mcp/src/main.rs (ares_simulate tool).
