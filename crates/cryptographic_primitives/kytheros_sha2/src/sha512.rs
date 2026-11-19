//! Incremental SHA-512 hashing, [NIST FIPS 180-4 (August 2015)][fips].
//!
//! [`BitSpan`] validates arbitrary-bit input in slice byte order, MSb-first
//! within each byte (§3.1). [`Sha512Context`] initializes the eight chaining
//! words (§5.3.5), compresses each complete 1024-bit block (§§5.2.2, 6.4.2),
//! and applies §5.1.2 padding with the original 128-bit bit length.
//! Finalization consumes the object and returns 64 bytes (H_0 through H_7,
//! each big-endian). The six public logical functions implement §4.1.3.
//!
//! The portable safe Rust implementation uses `no_std` and no heap.
//! The context owns a 128-byte block buffer and borrows a caller-owned
//! 16-word (128-byte) schedule workspace. It retains no input references.
//! Under the Kytheros architecture, the MCL owns authoritative state and
//! decides when to commit a working copy. The caller manages reuse and
//! cleanup. Ending a borrow or consuming the context cannot establish
//! physical erasure of its storage or intermediate copies.
//!
//! Published known answers and independent arbitrary-bit fixtures provide
//! functional development evidence. The unit tests also cover all padding
//! remainders, split updates, and synthetic near-limit lengths. The manual
//! `tests/sha512_tests/unit_tests/context_reference_vectors.py` script
//! reproduces the local fixtures. See `docs/assurance/project-scope.md`
//! for the project assurance boundary and limitations.
//!
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4

mod bitpack;
mod compression;
mod constants;
mod context;
mod logical;
mod schedule;

pub use bitpack::{BitSpan, PackingError};
pub use context::{Sha512Context, Sha512Error};
pub use logical::{
    big_sigma_0, big_sigma_1, ch, maj, small_sigma_0, small_sigma_1,
};
