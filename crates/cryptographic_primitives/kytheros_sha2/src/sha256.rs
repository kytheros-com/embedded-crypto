//! Incremental SHA-256 hashing, NIST FIPS 180-4 (August 2015).
//!
//! # Implemented scope and structure
//!
//! This development implementation contains input-span validation and
//! single-block packing (`bitpack`), six logical word functions (`logical`),
//! a sixteen-word circular message schedule (`schedule`), round constants
//! (`constants`), single-block compression (`compression`), and incremental
//! hashing (`context`). [`Sha256Context`] initializes, updates, pads and
//! serializes SHA-256 digests. [`BitSpan`] and [`PackingError`] expose input
//! validation; packing and compression remain internal. The six logical
//! functions retain their public API. In §6.2.2, [`ch`], [`maj`],
//! [`big_sigma_0`] and [`big_sigma_1`] combine round working variables;
//! [`small_sigma_0`] and [`small_sigma_1`] expand the message schedule.
//! Complete SHA-224 hashing remains unimplemented; sharing logical word
//! functions does not provide SHA-224 initialization or digest output.
//!
//! [`Sha256Context::new`] sets the §5.3.3 initial value. Updates append the
//! validated span's logical bits and immediately compress complete blocks.
//! Length overflow is rejected before mutation. [`Sha256Context::finalize`]
//! consumes the working object, applies §5.1.1 padding with the original
//! 64-bit length, and returns 32 digest bytes in H_0..H_7 order (§6.2.2).
//! The context owns one block buffer and borrows sixteen schedule words
//! per operation; no caller input reference is retained. See its API for
//! the buffered-bit invariants and a complete executable example.
//!
//! # Representation
//!
//! Words are numeric `u32` values; the leftmost bit is most significant
//! (FIPS 180-4 §3.1). Bitstrings traverse bytes in slice order, MSb-first
//! within each byte, including partially occupied bytes. Schedule decoding
//! uses big-endian byte order (§5.2.1), independent of host endianness.
//! Schedule and compression additions wrap modulo 2^32 (§3.2).
//! Compression accepts eight chaining words in H_0 through H_7 order.
//! The portable implementation uses safe Rust, `no_std` and no heap
//! allocation; component documentation describes working-storage contracts.
//!
//! # Planned Kytheros integration
//!
//! Per `docs/assurance/architecture.md`, `docs/design/mcl-fsm.md` and
//! `docs/design/module-interfaces.md`, the MCL owns authoritative state
//! and commits successful working-copy updates. Primitives use authorized
//! internal caller-provided storage, never authoritative contexts or
//! external mailbox storage. The PIL supplies platform isolation and
//! zeroization mechanisms under MCL coordination. These are architectural
//! contracts, not FIPS 180-4 rules or protections implemented by this module.
//! Storage reuse and cleanup belong to the caller; scope exit does not
//! establish erasure of register, stack or compiler-generated copies.
//!
//! # Evidence limits and references
//!
//! Component tests exercise Boolean identities, sampled inverse maps,
//! packing contracts, schedule recurrence and compression behavior. Their
//! objectives, oracle dependencies and limitations are documented locally.
//! Context tests add published byte-oriented known answers, locally derived
//! arbitrary-bit reference values, all partial-block padding sizes, split
//! equivalence and synthetic near-limit length checks. The standalone
//! `context/reference_vectors.py` reproduces the local fixtures and checks
//! byte-aligned reference calculations with Python's `hashlib`.
//! These are functional development checks, not module self-tests or
//! evidence of constant-time execution, side-channel or fault resistance,
//! secure zeroization, formal verification or independent audit. CAVP
//! algorithm validation, FIPS 140-3/CMVP module validation and production
//! readiness have not been established. See
//! `docs/assurance/project-scope.md` for the project assurance scope.
//!
//! The normative algorithm reference is [FIPS 180-4 (August 2015)][fips]:
//! §§3.1–3.2, §4.1.2 equations (4.2)–(4.7), §4.2.2, §5.1.1, §5.2.1,
//! §5.3.3 and §6.2.2. Logical-function
//! examples are calculated illustrations, not NIST-published vectors.
//!
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4

mod bitpack;
mod compression;
mod constants;
mod context;
mod logical;
mod schedule;

// Preserve the existing public logical-function API.
pub use logical::{
    big_sigma_0, big_sigma_1, ch, maj, small_sigma_0, small_sigma_1,
};

// Private parent imports are available throughout the sha256 hierarchy.
use constants::SHA_256_ROUND_CONSTANTS;
use schedule::MessageSchedule;

// Public hashing inputs and coordinator; component modules stay private.
use bitpack::pack_bits;
pub use bitpack::{BitSpan, PackingError};
use compression::compress;
pub use context::{Sha256Context, Sha256Error};
