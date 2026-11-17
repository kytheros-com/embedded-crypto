//! SHA-256 logical functions, NIST FIPS 180-4.
//!
//! # Scope and algorithm decomposition
//!
//! This file implements the six functions shared by SHA-224 and SHA-256.
//!
//! # Word representation and function contract
//!
//! Each logical-function input and result is a `u32` word. As in
//! FIPS 180-4 §3.1, the most significant bit is the leftmost bit of the
//! represented bit string. Operations on word values are independent of
//! host byte order.
//!
//! In the definitions below, AND, XOR and NOT operate bitwise, ROTR rotates
//! a 32-bit word right, and SHR shifts right with zero fill (§3.2).
//! The six logical functions accept every `u32` value, return a word
//! deterministically, and have no error or panic conditions. They allocate
//! no storage on the heap, perform no I/O and retain no persistent state.
//!
//! # Verification evidence
//!
//! Tests rotate Boolean truth-table fixtures and combine butterfly
//! permutations with rotation to check positional behavior. Sigma tests sample
//! inverse-map consistency using 50,000 deterministic LFSR states per
//! function. Inverse matrices use row vectors over GF(2); their generation and
//! independent review remain undocumented. These samples are not exhaustive
//! proofs of the FIPS definitions. Test comments specify the generator and
//! oracles.

// 4.1.2 SHA-224 and SHA-256 Functions (32-bit words).

/// Selects each result bit from `y` or `z` according to `x`.
///
/// `Ch(x, y, z) = (x AND y) XOR (NOT x AND z)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.2).
///
/// The three inputs are working words. At each bit position, a set bit in
/// `x` selects the corresponding bit of `y`; a clear bit selects `z`.
/// Thus `Ch(x, y, y) = y`. Selection is independent at each bit position.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::ch;
///
/// assert_eq!(ch(0xffff_0000, 0x1234_5678, 0x9abc_def0), 0x1234_def0);
/// ```
#[inline]
pub const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// Returns the bitwise majority of three working words.
///
/// `Maj(x, y, z) = (x AND y) XOR (x AND z) XOR (y AND z)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.3).
///
/// Each result bit is set exactly when at least two corresponding input
/// bits are set. The result is invariant under permutations of the three
/// inputs, and `Maj(x, x, z) = x`.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::maj;
///
/// assert_eq!(maj(0xffff_0000, 0xff00_ff00, 0xf0f0_f0f0), 0xfff0_f000);
/// ```
#[inline]
pub const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// Computes the SHA-256 Σ₀ transformation of a word.
///
/// `Σ₀(x) = ROTR²(x) XOR ROTR¹³(x) XOR ROTR²²(x)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.4).
///
/// Input `x` is a 32-bit word, used as round working variable `a`
/// in §6.2.2; the result is the transformed 32-bit word.
/// This map is linear over GF(2): `f(x XOR y) = f(x) XOR f(y)` and
/// `f(0) = 0`. This is bit-vector linearity, not integer additivity.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::big_sigma_0;
///
/// assert_eq!(big_sigma_0(1), 0x4008_0400);
/// assert_eq!(big_sigma_0(0), 0);
/// ```
#[inline]
pub const fn big_sigma_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// Computes the SHA-256 Σ₁ transformation of a word.
///
/// `Σ₁(x) = ROTR⁶(x) XOR ROTR¹¹(x) XOR ROTR²⁵(x)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.5).
///
/// Input `x` is a 32-bit word, used as round working variable `e`
/// in §6.2.2; the result is the transformed 32-bit word.
/// This map is linear over GF(2): `f(x XOR y) = f(x) XOR f(y)` and
/// `f(0) = 0`. This is bit-vector linearity, not integer additivity.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::big_sigma_1;
///
/// assert_eq!(big_sigma_1(1), 0x0420_0080);
/// assert_eq!(big_sigma_1(0), 0);
/// ```
#[inline]
pub const fn big_sigma_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// Computes the SHA-256 σ₀ transformation of a word.
///
/// `σ₀(x) = ROTR⁷(x) XOR ROTR¹⁸(x) XOR SHR³(x)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.6).
///
/// Input `x` is a 32-bit word, used as message-schedule word `W[t - 15]`
/// in §6.2.2; the result is the transformed 32-bit word.
/// This map is linear over GF(2): `f(x XOR y) = f(x) XOR f(y)` and
/// `f(0) = 0`. This is bit-vector linearity, not integer additivity.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::small_sigma_0;
///
/// assert_eq!(small_sigma_0(1), 0x0200_4000);
/// assert_eq!(small_sigma_0(0), 0);
/// ```
#[inline]
pub const fn small_sigma_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// Computes the SHA-256 σ₁ transformation of a word.
///
/// `σ₁(x) = ROTR¹⁷(x) XOR ROTR¹⁹(x) XOR SHR¹⁰(x)`.
/// Defined in FIPS 180-4 §4.1.2, equation (4.7).
///
/// Input `x` is a 32-bit word, used as message-schedule word `W[t - 2]`
/// in §6.2.2; the result is the transformed 32-bit word.
/// This map is linear over GF(2): `f(x XOR y) = f(x) XOR f(y)` and
/// `f(0) = 0`. This is bit-vector linearity, not integer additivity.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha256::small_sigma_1;
///
/// assert_eq!(small_sigma_1(1), 0x0000_a000);
/// assert_eq!(small_sigma_1(0), 0);
/// ```
#[inline]
pub const fn small_sigma_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[cfg(test)]
#[path = "../../tests/sha256_tests/unit_tests/logical_unit_tests.rs"]
mod tests;
