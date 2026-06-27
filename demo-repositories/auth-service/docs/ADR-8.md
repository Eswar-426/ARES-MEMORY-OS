# ADR-8: Mitigating Timing Attacks on Authentication Endpoints

**Status:** Accepted
**Date:** 2024-05-12
**Owner:** Security Team

## Context
During the recent penetration test (Incident PR-143), security researchers discovered that our HMAC signature validation could be brute-forced using a timing attack. The standard string equality operator `==` short-circuits on the first mismatched byte, revealing the correct prefix length based on response latency.

## Decision
All sensitive comparisons (passwords, HMAC signatures, API keys) must use a constant-time comparison algorithm. 
We have implemented `constant_time_compare` in the `crypto` module to XOR the entire byte array regardless of matching state.

## Consequences
- Prevents latency-based timing attacks.
- Slightly higher CPU overhead for failed comparisons, which is acceptable for security endpoints.
