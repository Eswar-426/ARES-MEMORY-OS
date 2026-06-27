# ARES MemoryOS for VS Code

![ARES Quality Report](https://img.shields.io/badge/ARES_Evaluation-96.4%25-brightgreen)

ARES is an AI-powered engineering intelligence system that understands your codebase as a semantic graph, not just raw text. This VS Code extension acts as the primary interface to the ARES intelligence engines.

## The Problem
Traditional AI coding tools (like Copilot or Cursor) are incredibly good at writing local functions, but they fundamentally fail at **system architecture**. When you ask *"Why does this module exist?"* or *"What happens if I change this core database trait?"*, they guess based on keyword proximity.

## What ARES Understands
ARES parses your repository into a deterministic, queryable Knowledge Graph. It doesn't just read code—it extracts:
- Abstract Syntax Trees (ASTs)
- Module relationships
- Function call graphs
- Architectural Decision Records (ADRs)
- Markdown requirements
- Ownership metadata

## The Five Engines

ARES exposes its graph through five deterministic intelligence engines accessible directly from the Chat Webview in this extension:

1. **Why Exists**: Understand the exact architectural, security, or business requirement that led to a specific piece of code.
2. **Impact Analysis**: See the exact "blast radius" of a change across files, traits, modules, and deployment pipelines.
3. **Traceability**: Track a high-level requirement directly down to the specific functions and tests that implement it.
4. **Drift Analysis**: Automatically detect when your codebase violates a documented architectural rule.
5. **Simulation**: Ask "What if I delete this?" and get an instant, deterministic list of everything that will break before you write a single line of code.

## Quick Start

1. Install the ARES MemoryOS extension.
2. Ensure you have the `ares-cli` installed and run `ares-cli ingest` in your project root to generate the SQLite Knowledge Graph (`.ares/ares.db`).
3. Start the ARES MCP Server.
4. Open the ARES Chat Webview in your VS Code sidebar.
5. Ask questions like *"What happens if I change the PaymentProvider trait?"*

## Architecture

This extension uses the Model Context Protocol (MCP) to seamlessly communicate with the ARES Reasoning Engines and the underlying SQLite Knowledge Graph. All data stays local to your machine.

## License
MIT
