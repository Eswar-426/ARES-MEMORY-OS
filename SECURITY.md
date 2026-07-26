# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Please do not open a public GitHub issue for security vulnerabilities.

Preferred: use GitHub's private vulnerability reporting 
(Security tab → "Report a vulnerability").

Include a description of the issue, steps to reproduce, and potential impact.

This is a solo-maintained open source project — response times may vary, 
but every report is taken seriously and addressed as quickly as possible. 
Expect an initial response within 5 days.

## Scope

ARES runs entirely locally and never transmits repository data externally. 
Security concerns of particular interest:

- Arbitrary code execution during repository scanning/ingestion
- SQL injection in the SQLite graph store
- Path traversal in file system operations
- Unsafe deserialization in the MCP protocol handler

## Disclosure Policy

We ask for reasonable time to address confirmed vulnerabilities before 
public disclosure. Reporters will be credited (with permission) in release 
notes once a fix ships.
