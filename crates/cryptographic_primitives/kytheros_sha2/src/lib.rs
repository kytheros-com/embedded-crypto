#![no_std]

//! SHA-2 implementations for embedded and resource-constrained systems.
//!
//! Currently, SHA-256 input validation, single-block bit packing, internal
//! message-schedule expansion, block compression and logical word functions
//! in [`sha256`] are implemented. The word functions also apply to SHA-224.
//! Complete hashing remains a future stage.
//! The portable core uses `no_std` and no heap
//! allocation. See the module for word representation, integration design,
//! executable examples and the limits of development-test evidence.
//!
//! This crate is under active development and is not ready for production
//! use. It is not independently audited or CAVP/CMVP validated. Planned
//! MCL/PIL integration does not establish module validation.

#[cfg(test)]
extern crate std;

pub mod sha256;
