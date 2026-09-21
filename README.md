# Embedded Crypto

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

Initial development will focus on SHA-2 and SHA-3/SHAKE primitives.

## Contributions

This project is not currently accepting external code contributions.
Bug reports and technical observations are welcome, subject to the
guidelines in [CONTRIBUTING.md](CONTRIBUTING.md).

Security-sensitive reports should follow [SECURITY.md](SECURITY.md).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

Copyright 2026 Jason Worth Martin. Published through Kytheros.

