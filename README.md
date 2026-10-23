# Embedded Crypto
[![CI](https://github.com/kytheros-com/embedded-crypto/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kytheros-com/embedded-crypto/actions/workflows/ci.yml)

Cryptographic implementations for embedded and resource-constrained
systems, developed by Jason Worth Martin and published through Kytheros.

## Status

This project is in early development. Its implementations have not been
independently audited or validated for production use.

Do not use this software to protect sensitive or operational data.

## Design goals

- `no_std` compatibility
- Allocation-free core APIs
- Explicit and reviewable implementations
- Conformance testing against authoritative test vectors
- Careful treatment of side-channel behavior
- Support for embedded and resource-constrained targets
- Optional reference integration with a five-mailbox MCL-controlled
  logical interface and target-specific PIL protection mechanisms

Initial development will focus on SHA-2 and SHA-3/SHAKE primitives.

The planned reference integration uses separate fixed-memory mailboxes
for five module logical interface categories. The MCL mediates external
information flow; a platform-specific PIL enforces access permissions.
Primitive crates remain independent, operating on authorized internal
caller-provided buffers. These integration features are **designs, not
implemented or verified capabilities**. See
[architecture](docs/assurance/architecture.md) and
[interface contract](docs/design/module-interfaces.md).

## Contributions

This project is not currently accepting external code contributions.
Bug reports and technical observations are welcome, subject to the
guidelines in [CONTRIBUTING.md](CONTRIBUTING.md).

Security-sensitive reports should follow [SECURITY.md](SECURITY.md).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

Copyright 2026 Jason Worth Martin. Published through Kytheros.

