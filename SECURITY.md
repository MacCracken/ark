# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Ark, please report it responsibly.

**Do not open a public issue.** Instead, email the maintainer directly or use GitHub's private vulnerability reporting feature.

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- Acknowledgement within 48 hours
- Assessment and plan within 7 days
- Fix and disclosure within 30 days (or sooner)

## Security Design

Ark is designed with security as a primary concern:

- **Plan-based execution**: Ark generates `InstallPlan` instructions but does not execute them directly. Execution requires privilege escalation through `shakti`.
- **SHA-256 integrity verification**: Every installed package is tracked with checksums. The `PackageDb` can detect tampering, corruption, and missing files.
- **Transactional operations**: All package operations are wrapped in transactions with rollback capability, preventing partial installs from corrupting system state.
- **Dependency resolution delegation**: Ark delegates all dependency resolution to `nous`, avoiding reimplementation bugs.
- **Source verification**: Packages are tracked by source (system, marketplace, flutter) with source-specific trust policies.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
