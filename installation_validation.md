# ARES Installation Validation Guide

## Purpose
This document outlines the steps required to validate a successful ARES installation before public testing.

## Step 1: CLI Validation
1. Build the CLI binary: `cargo build --release -p ares-cli`
2. Verify discoverability: Run `ares --help`. Ensure `ingest`, `memory`, `governance`, and `simulate` are listed.
3. Check version: Run `ares --version`.

## Step 2: Ingestion Pipeline Validation
1. Navigate to a test repository.
2. Run `ares ingest .`
3. **Expected Result**: A `.ares/knowledge_graph.json` file is produced without panics. The CLI outputs the number of nodes and edges discovered.

## Step 3: MCP Server Validation
1. Run the MCP Server: `cargo run -p ares-mcp`
2. Verify startup: The server should log `Starting ARES MCP Server` to stderr.
3. Send a test JSON-RPC request to invoke `ares_why_exists` and verify a valid JSON response is returned (no "Failed to serialize" strings).

## Step 4: Extension Validation
1. Navigate to `extensions/ares-memory-vscode`.
2. Run `npm install` and `npm run compile`.
3. Open the extension in VS Code Extension Development Host.
4. Verify the `ARES Memory` sidebar appears.
5. Invoke `ARES: Generate Snapshot` and verify the success notification.

*Note: Once gap analysis items are fixed, validation should also include triggering ingestion and MCP tools directly from the IDE.*
