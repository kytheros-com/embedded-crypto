//! SHA-512 components from [NIST FIPS 180-4 (August 2015)][fips].
//!
//! The module exposes six logical word functions from §4.1.3,
//! equations (4.8)–(4.13). These functions are stateless and allocation-free
//! in safe Rust for use in `no_std` environments. The API takes numeric
//! 64-bit words by value with the most significant bit at the left (§3.1).
//! It operates independently of host byte order and performs no
//! serialization.
//!
//! Internal components implement the round constants (§4.2.3), message
//! schedule and single-block compression (§6.4.2) for a future context.
//!
//! The examples are locally calculated illustrations. Unit tests check
//! Boolean behavior, sigma bit positions and sampled algebraic properties.
//!
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SHA-512 context will consume this compression module"
    )
)]
mod compression;
mod constants;
mod logical;
mod schedule;

pub use logical::{
    big_sigma_0, big_sigma_1, ch, maj, small_sigma_0, small_sigma_1,
};
