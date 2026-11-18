//! SHA-512 logical word functions from [NIST FIPS 180-4 (August 2015)][fips],
//! §4.1.3, equations (4.8)–(4.13).
//!
//! The module exposes six stateless, allocation-free functions in safe Rust
//! for use in `no_std` environments. The API takes numeric 64-bit words by
//! value, with the most significant bit at the left (§3.1). It operates
//! independently of host byte order and performs no serialization.
//!
//! The examples are locally calculated illustrations. Unit tests check
//! Boolean behavior, sigma bit positions and sampled algebraic properties.
//!
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4

mod logical;

pub use logical::{
    big_sigma_0, big_sigma_1, ch, maj, small_sigma_0, small_sigma_1,
};
