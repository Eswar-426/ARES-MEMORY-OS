# ARES Memory OS Architecture

## Overview
ARES Memory OS employs a highly modular, decoupled Rust architecture. It integrates SQLite databases (`ares-store`) for persistence, multi-provider LLM routing, and robust multi-agent coordination systems.

### Core Modules
1. **`ares-agent-runtime`**: Evaluates, reflects, and autonomously improves agent performance using continuous learning.
2. **`ares-memory-intelligence`**: Manages the extraction, clustering, and semantic compression of raw experiences into generalized principles.
3. **`ares-world-model`**: A discrete-event deterministic simulator allowing agents to test hypotheses in safe sandbox environments.
4. **`ares-coordination`**: Supports debate, consensus voting, swarm execution, and dynamic organizational hierarchies.
5. **`ares-model-testing`**: Fallback and capability assessment system for dynamically selecting the best LLM provider (NVIDIA, Groq, OpenRouter, Google).
6. **`ares-api`**: An Axum-based API providing telemetry, configuration, and control to external dashboards and clients.
