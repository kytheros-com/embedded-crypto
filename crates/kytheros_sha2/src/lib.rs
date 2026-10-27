#![no_std]

//! SHA-2 implementations for embedded and resource-constrained systems.
//!
//! Currently, only the six SHA-256 logical word functions in [`sha256`]
//! are implemented. They are also used by SHA-224; neither complete hash
//! algorithm is available. The portable core uses `no_std` and no heap
//! allocation. See the module for word representation, integration design,
//! executable examples and the limits of development-test evidence.
//!
//! This crate is under active development and is not ready for production
//! use. It is not independently audited or CAVP/CMVP validated. Planned
//! MCL/PIL integration does not establish module validation.

pub mod sha256;
