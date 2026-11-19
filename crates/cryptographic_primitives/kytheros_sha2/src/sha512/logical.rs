//! SHA-512 logical functions, NIST FIPS 180-4 (August 2015).
//!
//! Inputs and outputs are numeric `u64` words with the most significant bit
//! at the left (§3.1). Word operations are independent of host byte order.
//! AND, XOR and NOT act bitwise. ROTR rotates a full 64-bit word right and
//! SHR shifts right with zero fill (§3.2).
//!
//! Every input word is valid. Each function returns a deterministic result
//! without panicking, allocating, performing I/O or retaining state.

/// Selects each result bit from `y` or `z` according to `x`.
///
/// `Ch(x, y, z) = (x AND y) XOR (NOT x AND z)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.8).
///
/// The three inputs are working words. At each bit position, a set bit in
/// `x` selects the corresponding bit of `y`; a clear bit selects `z`.
/// Thus `Ch(x, y, y) = y`. Selection is independent at each bit position.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::ch;
///
/// let high_half = 0xffff_ffff_0000_0000;
/// assert_eq!(ch(high_half, u64::MAX, 0), high_half);
/// ```
#[inline]
pub const fn ch(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (!x & z)
}

/// Returns the bitwise majority of three working words.
///
/// `Maj(x, y, z) = (x AND y) XOR (x AND z) XOR (y AND z)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.9).
///
/// Each result bit is set exactly when at least two corresponding input
/// bits are set. The result is invariant under permutations of the three
/// inputs, and `Maj(x, x, z) = x`.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::maj;
///
/// let alternating = 0xaaaa_aaaa_aaaa_aaaa;
/// assert_eq!(maj(0, alternating, u64::MAX), alternating);
/// ```
#[inline]
pub const fn maj(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// Computes the SHA-512 Σ₀ transformation of a word.
///
/// `Σ₀(x) = ROTR²⁸(x) XOR ROTR³⁴(x) XOR ROTR³⁹(x)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.10).
///
/// Input `x` is a 64-bit word, used as round working variable `a`
/// in §6.4.2; the result is the transformed 64-bit word.
/// This map is linear over GF(2), with XOR as vector addition.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::big_sigma_0;
///
/// assert_eq!(big_sigma_0(1), 0x0000_0010_4200_0000);
/// assert_eq!(big_sigma_0(0), 0);
/// ```
#[inline]
pub const fn big_sigma_0(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

/// Computes the SHA-512 Σ₁ transformation of a word.
///
/// `Σ₁(x) = ROTR¹⁴(x) XOR ROTR¹⁸(x) XOR ROTR⁴¹(x)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.11).
///
/// Input `x` is a 64-bit word, used as round working variable `e`
/// in §6.4.2; the result is the transformed 64-bit word.
/// This map is linear over GF(2), with XOR as vector addition.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::big_sigma_1;
///
/// assert_eq!(big_sigma_1(1), 0x0004_4000_0080_0000);
/// assert_eq!(big_sigma_1(0), 0);
/// ```
#[inline]
pub const fn big_sigma_1(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

/// Computes the SHA-512 σ₀ transformation of a word.
///
/// `σ₀(x) = ROTR¹(x) XOR ROTR⁸(x) XOR SHR⁷(x)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.12).
///
/// Input `x` is a 64-bit word, used as message-schedule word `W[t - 15]`
/// in §6.4.2; the result is the transformed 64-bit word.
/// This map is linear over GF(2), with XOR as vector addition.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::small_sigma_0;
///
/// assert_eq!(small_sigma_0(1), 0x8100_0000_0000_0000);
/// assert_eq!(small_sigma_0(0), 0);
/// ```
#[inline]
pub const fn small_sigma_0(x: u64) -> u64 {
    x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7)
}

/// Computes the SHA-512 σ₁ transformation of a word.
///
/// `σ₁(x) = ROTR¹⁹(x) XOR ROTR⁶¹(x) XOR SHR⁶(x)`.
/// Defined in FIPS 180-4 §4.1.3, equation (4.13).
///
/// Input `x` is a 64-bit word, used as message-schedule word `W[t - 2]`
/// in §6.4.2; the result is the transformed 64-bit word.
/// This map is linear over GF(2), with XOR as vector addition.
///
/// # Examples
///
/// ```rust
/// use kytheros_sha2::sha512::small_sigma_1;
///
/// assert_eq!(small_sigma_1(1), 0x0000_2000_0000_0008);
/// assert_eq!(small_sigma_1(0), 0);
/// ```
#[inline]
pub const fn small_sigma_1(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
}

#[cfg(test)]
#[path = "../../tests/sha512_tests/unit_tests/logical_unit_tests.rs"]
mod tests;
