# Getting Started with ARES

## Prerequisites
- Rust (stable)
- Node.js 20+ (for the Dashboard)
- SQLite3

## Installation
1. Clone the repository.
2. Run `cargo build --release` to build the core services.
3. Start the API server: `cargo run -p ares-api`

## Environment Setup
Create a `.env` file based on `.env.example`. Key variables include:
- `NVIDIA_API_KEY`
- `OPENROUTER_API_KEY`
- `GEMINI_API_KEY`
- `GROQ_API_KEY`
- `ARES_TELEMETRY_DB_PATH` (defaults to `.ares/ares.db`)

## Running the Telemetry Dashboard
Navigate to `apps/dashboard`:
```bash
cd apps/dashboard
npm install
npm run dev
```
