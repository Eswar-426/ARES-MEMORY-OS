# SEC-4: Secret Comparison Standards

**Category**: Security Requirement
**Owner**: Security Team

Any code comparing secrets (tokens, keys, passwords, hashes, HMACs) must be completely immune to timing attacks. Under no circumstances should standard string equality (`==` or `.equals()`) be used for secret validation in the `auth-service`. 

See `ADR-8` for implementation specifics.
