# Evidence: Streaming Benchmark Results
## ID: EVD-streaming-benchmark

This evidence supports ADR-003 and ADR-012.

The streaming benchmark results show that the batched ingestion model significantly reduces peak RSS consumption on massive repositories like Next.js (from 173MB to 71.2MB), proving that bounded memory is achieved while dropping ingestion time from 106s to 21.9s.

Detailed metrics are available in `reports/validation/memory_breakdown.md`.
