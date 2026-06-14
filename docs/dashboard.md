# ARES Telemetry Dashboard

The ARES Telemetry Dashboard is a React + Vite + TailwindCSS application that provides real-time visualization of agent runs, benchmark metrics, and model provider health.

## Features
- **Real-Time Telemetry**: Connects to `ares-api` to poll `/api/v1/telemetry/latest`.
- **Dynamic Fallback Visualization**: Displays the exact model chains selected for Architecture, Feature, and Debug roles based on latency and capability scores.
- **Provider Health Monitoring**: Displays the current status (Healthy, Degraded, RateLimited) of configured LLM providers.

## Configuration
The dashboard automatically connects to the backend running on `http://localhost:3000`. No additional configuration is required.
