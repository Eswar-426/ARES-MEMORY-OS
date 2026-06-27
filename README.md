# ARES MemoryOS

![ARES Quality Report](https://img.shields.io/badge/ARES_Evaluation-96.4%25-brightgreen)
![Stability](https://img.shields.io/badge/Determinism-100%25-blue)

ARES is an AI-powered engineering intelligence system that understands your codebase as a semantic graph, not just raw text.

## The Problem
Modern development moves fast. When a Staff Engineer leaves, the context leaves with them. Traditional AI coding tools (like Copilot or Cursor) are incredibly good at writing local functions, but they fundamentally fail at **system architecture**. 

They operate on unstructured chunks of text. When you ask *"Why does this module exist?"* or *"What happens if I change this core database trait?"*, they guess based on keyword proximity.

## What ARES Understands
ARES parses your repository into a deterministic, queryable Knowledge Graph. It doesn't just read code—it extracts:
- Abstract Syntax Trees (ASTs)
- Module relationships
- Function call graphs
- Architectural Decision Records (ADRs)
- Markdown requirements
- Ownership metadata

## The Five Engines

ARES exposes its graph through five deterministic intelligence engines accessible via our VS Code extension:

1. **Why Exists**: Understand the exact architectural, security, or business requirement that led to a specific piece of code.
2. **Impact Analysis**: See the exact "blast radius" of a change across files, traits, modules, and deployment pipelines.
3. **Traceability**: Track a high-level requirement (e.g. `REQ-12`) directly down to the specific functions and tests that implement it.
4. **Drift Analysis**: Automatically detect when your codebase violates a documented architectural rule (e.g., bypassing a repository layer).
5. **Simulation**: Ask "What if I delete this?" and get an instant, deterministic list of everything that will break before you write a single line of code.

## Quick Start

Experience ARES in 5 minutes using our built-in demo orchestration script.

### 1. Prerequisites
- Rust and Cargo (`rustup default stable`)
- VS Code (`code` command in PATH)

### 2. Run the Demo
Launch the demo script from the repository root:
```powershell
./demo.ps1
```

By default, this sets up the **Payment Service** demo. You can also target specific scenarios:
```powershell
./demo.ps1 payment-service   # Impact & Traceability
./demo.ps1 inventory-system  # Architecture Drift
./demo.ps1 auth-service      # Why Exists
```

### 3. Ask "Wow" Questions
Once VS Code opens, use the ARES Chat Webview to ask:
- *"What happens if I change the PaymentProvider trait?"*
- *"Show me everything implementing REQ-12."*
- *"Are there any architecture violations of ADR-3?"*

## Architecture

```text
crates/ares-core       -> Core Graph Data Structures
crates/ares-store      -> Immutable SQLite Persistence (ares.db)
crates/ares-scanner    -> Multi-language parser (Rust, JS/TS, Markdown)
crates/ares-reasoning  -> The Five Intelligence Engines
crates/ares-mcp        -> Model Context Protocol Server
crates/ares-cli        -> High-speed CLI (ingest, benchmark, doctor)
extensions/            -> VS Code Extension & Webview UI
```

## Evaluation Platform: Engineering Quality Platform

ARES includes a world-class, **deterministic Evaluation Harness** (`evaluation/`). Unlike standard LLM benchmarks which are notoriously flaky, ARES converts all intelligence outputs into a **Versioned Canonical Fact Model** mapped to strict graph node IDs, producing mathematically verifiable scores across:

- **Recall & Precision**
- **Evidence Coverage**
- **Hallucination Penalties**
- **SHA-256 Stability Fingerprinting**

Run the evaluator:
```bash
cargo run --bin ares-evaluation -- run --dataset evaluation/datasets/ares/cases.json --repo .
```
Compare regressions:
```bash
cargo run --bin ares-evaluation -- compare --latest 2026-06-27_16-15-08 --previous 2026-06-27_16-08-04
```

## License
MIT
