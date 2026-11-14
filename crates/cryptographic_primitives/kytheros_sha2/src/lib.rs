#![no_std]

//! SHA-2 implementations for embedded and resource-constrained systems.
//!
//! [`sha256::Sha256Context`] implements incremental SHA-256 hashing with
//! arbitrary-bit updates, FIPS 180-4 padding and big-endian digest output.
//! The word functions also apply to SHA-224; SHA-224 hashing is not
//! implemented. The portable core uses safe Rust, `no_std` and no heap
//! allocation. Callers supply sixteen schedule words per operation. See
//! [`sha256`] for representation, ownership, examples and evidence limits.
//!
//! This crate is under active development and is not ready for production
//! use. It is not independently audited or CAVP/CMVP validated. Planned
//! MCL/PIL integration does not establish module validation.

#[cfg(test)]
extern crate std;

pub mod sha256;
