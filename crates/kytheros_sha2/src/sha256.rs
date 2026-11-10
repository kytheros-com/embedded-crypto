//! SHA-256 input packing and block compression, NIST FIPS 180-4.
//!
//! # Scope and algorithm decomposition
//!
//! This draft implements input-span validation, single-block bit packing,
//! message-schedule expansion, block compression, and six functions shared by
//! SHA-224 and SHA-256.
//! In SHA-256 hash computation (§6.2.2), [`ch`], [`maj`],
//! [`big_sigma_0`] and [`big_sigma_1`] combine working variables in each
//! round; [`small_sigma_0`] and [`small_sigma_1`] expand the message
//! schedule. The internal schedule decodes sixteen big-endian words and
//! yields W_0 through W_63 using caller-owned circular working storage.
//! Compression takes H_0 through H_7 as eight numeric 32-bit words,
//! performs 64 rounds, and adds the final working words to the original
//! chaining words modulo 2^32. It uses 16 caller-provided schedule words
//! and eight temporary working words without dynamic allocation.
//! Padding, hash initialization and digest output remain future stages.
//!
//! # Word representation and function contract
//!
//! Each logical-function input and result is a `u32` word. As in
//! FIPS 180-4 §3.1, the most significant bit is the leftmost bit of the
//! represented bit string. Operations on word values are independent of
//! host byte order. [`BitSpan`] and [`pack_bits`] traverse bytes in slice
//! order and bits MSb-first. Schedule decoding explicitly uses big-endian
//! byte order; schedule sums wrap modulo 2^32 (FIPS 180-4 §3.2).
//!
//! In the definitions below, AND, XOR and NOT operate bitwise, ROTR rotates
//! a 32-bit word right, and SHR shifts right with zero fill (§3.2).
//! The six logical functions accept every `u32` value, return a word
//! deterministically,
//! and have no error or panic conditions. They allocate no storage on the
//! heap, perform no I/O and retain no persistent internal state.
//!
//! # Kytheros integration design
//!
//! The planned Module Control Layer (MCL) owns authoritative cryptographic
//! state and commits working-copy updates. Primitives use temporary word
//! values or caller-provided working storage; they neither access nor
//! commit MCL state. MCL
//! ownership is a Kytheros architecture decision, not a FIPS 180-4 rule.
//! The MCL and platform protection mechanisms are not implemented here.
//! Lack of retained state does not establish erasure of register or stack
//! copies, nor does it establish side-channel or fault resistance.
//!
//! # Evidence limits and references
//!
//! This is an incomplete development implementation, not production-ready
//! software. Unit tests exercise Boolean identities and sampled inverse-map
//! consistency. Packing tests check alignment, boundaries and preservation
//! against local text fixtures. Schedule tests compare a direct 64-word
//! reference and independent arithmetic fixtures (see the schedule module).
//! Compression tests check a direct reference, round checkpoints, workspace
//! reuse and a [NIST NSRL][nsrl] single-block known answer. These are sampled
//! functional development checks.
//! No independent audit, formal verification, CAVP algorithm validation or
//! FIPS 140-3/CMVP module validation is claimed.
//!
//! The normative algorithm reference is [FIPS 180-4 (August 2015)][fips].
//! Function definitions follow §4.1.2, equations (4.2)–(4.7). Examples below
//! are independently calculated illustrations, not NIST-published vectors.
//!
//! [nsrl]:
//! https://nist.gov/itl/ai/ai-standards-and-guidelines-group/nsrl-test-data
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4

pub(crate) mod compression;
pub(crate) mod schedule;

/// A validated borrowed bitstring, traversed MSb-first within each byte.
///
/// Logical bit `i` is bit `7 - ((offset_in_bits + i) % 8)` of byte
/// `(offset_in_bits + i) / 8`. Bytes occur in slice order, independent of
/// native endianness (FIPS 180-4 §3.1). No input bytes are copied.
/// Fields are private so construction preserves the validated bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitSpan<'a> {
    input: &'a [u8],
    offset_in_bits: usize,
    len_in_bits: usize,
}

/// Rejection reasons for bitstring construction and block packing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackingError {
    /// The slice capacity or source endpoint overflowed `usize`.
    ArithmeticOverflow,
    /// The source endpoint exceeds the slice's bit capacity.
    SourceOutOfRange,
    /// The destination position exceeds 512 bits.
    DestinationOutOfRange,
}

impl<'a> BitSpan<'a> {
    /// Validates the entire span before allowing any use by the packer.
    ///
    /// Both `input.len() * 8` and `offset_in_bits + len_in_bits` use
    /// checked arithmetic. Overflow returns
    /// [`PackingError::ArithmeticOverflow`];
    /// an endpoint beyond capacity returns [`PackingError::SourceOutOfRange`].
    /// A zero-length span may start anywhere through the capacity,
    /// inclusive; an empty slice permits only offset zero and length zero.
    /// This operation does not panic, allocate, or modify storage.
    pub fn new(
        input: &'a [u8],
        offset_in_bits: usize,
        len_in_bits: usize,
    ) -> Result<Self, PackingError> {
        let capacity_in_bits = input
            .len()
            .checked_mul(8)
            .ok_or(PackingError::ArithmeticOverflow)?;
        let end_in_bits = offset_in_bits
            .checked_add(len_in_bits)
            .ok_or(PackingError::ArithmeticOverflow)?;
        if end_in_bits > capacity_in_bits {
            return Err(PackingError::SourceOutOfRange);
        }
        Ok(Self {
            input,
            offset_in_bits,
            len_in_bits,
        })
    }
}

/// Progress from one call to [`pack_bits`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackingProgress {
    /// Number of leading logical source bits written by this call.
    pub consumed_in_bits: usize,
    /// First unwritten destination position, in the inclusive range 0..=512.
    pub destination_position_in_bits: usize,
}

/// Copies a validated bitstring into one caller-owned 512-bit block.
///
/// With initial destination position `d`, writes exactly
/// `n = min(source.len_in_bits, 512 - d)` bits to `[d, d + n)` and
/// preserves every other bit. Source and destination use MSb-first bit
/// order and slice byte order, following FIPS 180-4 §3.1. The block size
/// follows §5.2.1; this function does not parse blocks into words.
///
/// Positions 0..=512 are accepted. Position 512 returns zero consumption
/// without modifying the full block. The caller must process the block
/// before choosing a new position for reuse; this stateless API cannot
/// enforce that lifecycle. It never clears or automatically reuses a block.
/// Empty input also succeeds without modification at every valid position.
///
/// # Errors
///
/// A position above 512 returns [`PackingError::DestinationOutOfRange`]
/// before any write. Invalid source spans cannot be constructed through
/// [`BitSpan::new`]. All rejections leave destination storage unchanged.
///
/// # Invariants and scope
///
/// Before iteration `i`, exactly `[d, d + i)` has been copied. Validated
/// source bounds and `i < n` bound every index and addition; no input can
/// cause arithmetic overflow or an out-of-bounds panic. At most 512 loop
/// iterations run, using scalar temporaries and no additional block buffer.
/// No allocation, native-endian conversion, MCL access, or retained state
/// is involved. The caller owns all working storage and cleanup duties.
/// This is input packing only: no padding or other hashing stage is done.
pub fn pack_bits(
    source: BitSpan<'_>,
    block: &mut [u8; 64],
    destination_offset_in_bits: usize,
) -> Result<PackingProgress, PackingError> {
    if destination_offset_in_bits > 512 {
        return Err(PackingError::DestinationOutOfRange);
    }
    let consumed_in_bits =
        source.len_in_bits.min(512 - destination_offset_in_bits);
    for i in 0..consumed_in_bits {
        let source_position_in_bits = source.offset_in_bits + i;
        let destination_position_in_bits = destination_offset_in_bits + i;
        let bit = (source.input[source_position_in_bits / 8]
            >> (7 - source_position_in_bits % 8))
            & 1;
        let mask = 1 << (7 - destination_position_in_bits % 8);
        let byte = &mut block[destination_position_in_bits / 8];
        *byte = (*byte & !mask) | (bit * mask);
    }
    Ok(PackingProgress {
        consumed_in_bits,
        destination_position_in_bits: destination_offset_in_bits
            + consumed_in_bits,
    })
}

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
mod tests {
    use super::*;

    const WORD_SIZE_IN_BITS: u32 = 32;
    // Local Boolean fixtures: bits 12..15 encode all eight input triples
    // together with bits 16..19. Expected words follow the truth tables;
    // no NIST-published vector source is recorded for these values.
    const TEST_X: u32 = 0x0000f000;
    const TEST_Y: u32 = 0x00033000;
    const TEST_Z: u32 = 0x00055000;

    const CH_ANSWERS: u32 = 0x00053000;
    const MAJ_ANSWERS: u32 = 0x00017000;

    const SIGMA_BULK_TEST_NUM_ITERATIONS: u32 = 50_000;

    // Inverse matrices over GF(2), using row-vector multiplication.
    // Entry i encodes row i: the image of basis vector 1 << i under the
    // inverse transformation.
    // They are test oracles, not FIPS constants or NIST-published vectors.
    // Their generation method and independent review remain undocumented.
    const BIG_SIGMA_0_INVERSE: [u32; 32] = [
        0xCBD1A68D, 0x97A34D1B, 0x2F469A37, 0x5E8D346E, 0xBD1A68DC, 0x7A34D1B9,
        0xF469A372, 0xE8D346E5, 0xD1A68DCB, 0xA34D1B97, 0x469A372F, 0x8D346E5E,
        0x1A68DCBD, 0x34D1B97A, 0x69A372F4, 0xD346E5E8, 0xA68DCBD1, 0x4D1B97A3,
        0x9A372F46, 0x346E5E8D, 0x68DCBD1A, 0xD1B97A34, 0xA372F469, 0x46E5E8D3,
        0x8DCBD1A6, 0x1B97A34D, 0x372F469A, 0x6E5E8D34, 0xDCBD1A68, 0xB97A34D1,
        0x72F469A3, 0xE5E8D346,
    ];

    const BIG_SIGMA_1_INVERSE: [u32; 32] = [
        0x6AB84F6C, 0xD5709ED8, 0xAAE13DB1, 0x55C27B63, 0xAB84F6C6, 0x5709ED8D,
        0xAE13DB1A, 0x5C27B635, 0xB84F6C6A, 0x709ED8D5, 0xE13DB1AA, 0xC27B6355,
        0x84F6C6AB, 0x09ED8D57, 0x13DB1AAE, 0x27B6355C, 0x4F6C6AB8, 0x9ED8D570,
        0x3DB1AAE1, 0x7B6355C2, 0xF6C6AB84, 0xED8D5709, 0xDB1AAE13, 0xB6355C27,
        0x6C6AB84F, 0xD8D5709E, 0xB1AAE13D, 0x6355C27B, 0xC6AB84F6, 0x8D5709ED,
        0x1AAE13DB, 0x355C27B6,
    ];

    const SMALL_SIGMA_0_INVERSE: [u32; 32] = [
        0x185744E9, 0x30AE89D2, 0x615D13A4, 0xDAED63A1, 0x9CD03A8E, 0x08FDCC39,
        0x11FB9872, 0x23F730E4, 0x5FB92521, 0xBF724A42, 0x57EE6948, 0xAFDCD290,
        0x76B358EC, 0xF531F531, 0xC36917AE, 0xB78F9679, 0x4615D13E, 0x947CE695,
        0x19A4740F, 0x2B1FACF7, 0x4E681D07, 0x84877EE7, 0x385344EB, 0x70A689D6,
        0xF91A5745, 0xC36917AF, 0xB78F967B, 0x4615D13A, 0x8C2BA274, 0x290AFDCD,
        0x4A42BF73, 0x94857EE6,
    ];

    const SMALL_SIGMA_1_INVERSE: [u32; 32] = [
        0x2CCFFFED, 0x75500037, 0xEAA0006E, 0x589D5570, 0xB13AAAE0, 0xC367FF81,
        0x27DD5543, 0x4FBAAA86, 0xB3BAAAE1, 0xC667FF83, 0x2DDD5547, 0x5BBAAA8E,
        0x9BBAAAF1, 0x9667FFA3, 0x8DDD5507, 0x9667FFA2, 0x8DDD5505, 0x9667FFA6,
        0x8DDD550D, 0x9667FFB6, 0x8DDD552D, 0x9667FFF6, 0x8DDD55AD, 0x9667FEF6,
        0x8DDD57AD, 0xBAA8051B, 0xF88D5F9A, 0x50081575, 0xA0102AEA, 0xE132FF95,
        0x6377556B, 0xC6EEAAD6,
    ];

    // Right-shifting Galois LFSR: XOR this mask after shifting exactly
    // when the old least significant bit is one. Set mask bits are
    // positions 31, 21, 1 and 0 (LSB = position 0). This convention avoids
    // ambiguity with reciprocal-polynomial or left-shifting descriptions.
    // Zero is absorbing; tests start at 0x8000_0000. No maximal-period
    // or statistical-randomness claim is needed for these sampled tests.
    const LFSR_MASK: u32 = 0x8020_0003;

    #[inline]
    fn lfsr_next(x: u32) -> u32 {
        (x >> 1) ^ (0_u32.wrapping_sub(x & 1) & LFSR_MASK)
    }

    // Reverse bit order by swapping halves, then groups of 8, 4, 2, 1.
    // Each stage uses complementary masks to select the exchanged groups.
    // Applied to every operand and answer, any common bit permutation
    // preserves Ch/Maj because they act independently at each position.
    #[inline]
    fn butterfly_permutation(x: u32) -> u32 {
        let mut shift = 16;
        let mut mask: u32 = 0xffff_0000;
        let mut y = x;
        while shift > 0 {
            y = ((y << shift) & mask) | ((y >> shift) & (!mask));
            shift >>= 1;
            mask ^= mask >> shift;
        }
        y
    }

    // Rotate truth-table fixtures through all bit positions, then mix
    // reversal with rotation to exercise a second positional arrangement.
    // Checks bitwise Boolean behavior; not exhaustive over word triples
    // and not a test of complete SHA-256 compression or hashing.
    #[test]
    fn ch_walking_bits_test() {
        let mut x = TEST_X;
        let mut y = TEST_Y;
        let mut z = TEST_Z;
        let mut ans = CH_ANSWERS;
        for _ in 0..WORD_SIZE_IN_BITS {
            assert_eq!(ch(x, y, z), ans);
            x = x.rotate_right(1);
            y = y.rotate_right(1);
            z = z.rotate_right(1);
            ans = ans.rotate_right(1);
        }
        for _ in 0..WORD_SIZE_IN_BITS {
            x = butterfly_permutation(x);
            y = butterfly_permutation(y);
            z = butterfly_permutation(z);
            ans = butterfly_permutation(ans);
            assert_eq!(ch(x, y, z), ans);
            x = x.rotate_right(1);
            y = y.rotate_right(1);
            z = z.rotate_right(1);
            ans = ans.rotate_right(1);
        }
    }

    // Rotate truth-table fixtures through all bit positions, then mix
    // reversal with rotation to exercise a second positional arrangement.
    // Checks bitwise Boolean behavior; not exhaustive over word triples
    // and not a test of complete SHA-256 compression or hashing.
    #[test]
    fn maj_walking_bits_test() {
        let mut x = TEST_X;
        let mut y = TEST_Y;
        let mut z = TEST_Z;
        let mut ans = MAJ_ANSWERS;
        for _ in 0..WORD_SIZE_IN_BITS {
            assert_eq!(maj(x, y, z), ans);
            x = x.rotate_right(1);
            y = y.rotate_right(1);
            z = z.rotate_right(1);
            ans = ans.rotate_right(1);
        }
        for _ in 0..WORD_SIZE_IN_BITS {
            x = butterfly_permutation(x);
            y = butterfly_permutation(y);
            z = butterfly_permutation(z);
            ans = butterfly_permutation(ans);
            assert_eq!(maj(x, y, z), ans);
            x = x.rotate_right(1);
            y = y.rotate_right(1);
            z = z.rotate_right(1);
            ans = ans.rotate_right(1);
        }
    }

    // Multiply a row vector x by a row-encoded 32 x 32 binary matrix.
    // Bit i of x selects row a[i]; XOR is addition over GF(2).
    // The signed right shift expands the selected bit into a full mask.
    fn apply_matrix(x: u32, a: [u32; 32]) -> u32 {
        let mut results: u32 = 0;
        for (i, row) in a.iter().enumerate() {
            results ^= ((((x << (31 - i)) as i32) >> 31) as u32) & row;
        }
        results
    }

    // Sample inverse(f(x)) = x for 50,000 deterministic LFSR states.
    // OR accumulates every mismatch bit so failures cannot cancel.
    // Consistency depends on the inverse oracle: these samples do not
    // independently prove the FIPS formula, exhaustive coverage or period.
    #[test]
    fn big_sigma_0_test() {
        let mut x: u32 = 0x8000_0000;
        let mut result: u32 = 0;
        for _ in 0..SIGMA_BULK_TEST_NUM_ITERATIONS {
            result |= apply_matrix(big_sigma_0(x), BIG_SIGMA_0_INVERSE) ^ x;
            x = lfsr_next(x);
        }
        assert_eq!(result, 0);
    }

    // Sample inverse(f(x)) = x for 50,000 deterministic LFSR states.
    // OR accumulates every mismatch bit so failures cannot cancel.
    // Consistency depends on the inverse oracle: these samples do not
    // independently prove the FIPS formula, exhaustive coverage or period.
    #[test]
    fn big_sigma_1_test() {
        let mut x: u32 = 0x8000_0000;
        let mut result: u32 = 0;
        for _ in 0..SIGMA_BULK_TEST_NUM_ITERATIONS {
            result |= apply_matrix(big_sigma_1(x), BIG_SIGMA_1_INVERSE) ^ x;
            x = lfsr_next(x);
        }
        assert_eq!(result, 0);
    }

    // Sample inverse(f(x)) = x for 50,000 deterministic LFSR states.
    // OR accumulates every mismatch bit so failures cannot cancel.
    // Consistency depends on the inverse oracle: these samples do not
    // independently prove the FIPS formula, exhaustive coverage or period.
    #[test]
    fn small_sigma_0_test() {
        let mut x: u32 = 0x8000_0000;
        let mut result: u32 = 0;
        for _ in 0..SIGMA_BULK_TEST_NUM_ITERATIONS {
            result |= apply_matrix(small_sigma_0(x), SMALL_SIGMA_0_INVERSE) ^ x;
            x = lfsr_next(x);
        }
        assert_eq!(result, 0);
    }

    // Sample inverse(f(x)) = x for 50,000 deterministic LFSR states.
    // OR accumulates every mismatch bit so failures cannot cancel.
    // Consistency depends on the inverse oracle: these samples do not
    // independently prove the FIPS formula, exhaustive coverage or period.
    #[test]
    fn small_sigma_1_test() {
        let mut x: u32 = 0x8000_0000;
        let mut result: u32 = 0;
        for _ in 0..SIGMA_BULK_TEST_NUM_ITERATIONS {
            result |= apply_matrix(small_sigma_1(x), SMALL_SIGMA_1_INVERSE) ^ x;
            x = lfsr_next(x);
        }
        assert_eq!(result, 0);
    }
}

#[cfg(test)]
mod packing_tests {
    use super::*;

    // Local fixtures, not NIST vectors: the logical string is
    // 1011001101101101 (hex B36D), shifted right by each source offset.
    // Ones outside the span deliberately differ between representations.
    const SOURCES: [[u8; 3]; 8] = [
        [0xb3, 0x6d, 0xff],
        [0xd9, 0xb6, 0xff],
        [0xec, 0xdb, 0x7f],
        [0xf6, 0x6d, 0xbf],
        [0xfb, 0x36, 0xdf],
        [0xfd, 0x9b, 0x6f],
        [0xfe, 0xcd, 0xb7],
        [0xff, 0x66, 0xdb],
    ];
    const LOGICAL_BITS: &str = "1011001101101101";

    // An independent text oracle: concatenate textual bits and decode
    // complete groups with radix parsing, rather than source extraction
    // and destination masks. Comparing the entire block checks untouched
    // prefix/suffix bits, including partial bytes. Test-only allocation
    // belongs to the hosted harness, not to the portable implementation.
    fn assert_block_text(block: &[u8; 64], expected: &str) {
        assert_eq!(expected.len(), 512);
        for (byte_index, byte_text) in expected.as_bytes().chunks(8).enumerate()
        {
            let byte_text = core::str::from_utf8(byte_text).unwrap();
            let expected_byte = u8::from_str_radix(byte_text, 2).unwrap();
            assert_eq!(block[byte_index], expected_byte, "byte {byte_index}");
        }
    }

    // Covers all 8 x 8 alignments, lengths 1..=16, and both clear/set
    // sentinel bits. These finite fixtures are functional evidence only;
    // they do not prove correctness for all strings, timing, or full SHA-256.
    #[test]
    fn alignments_lengths_and_preservation() {
        for (source_offset_in_bits, input) in SOURCES.iter().enumerate() {
            for destination_offset_in_bits in 0..8 {
                for len_in_bits in 1..=16 {
                    for (sentinel, sentinel_text) in
                        [(0xa5, "10100101"), (0x5a, "01011010")]
                    {
                        let mut block = [sentinel; 64];
                        let source = BitSpan::new(
                            input,
                            source_offset_in_bits,
                            len_in_bits,
                        )
                        .unwrap();
                        let progress = pack_bits(
                            source,
                            &mut block,
                            destination_offset_in_bits,
                        )
                        .unwrap();
                        assert_eq!(progress.consumed_in_bits, len_in_bits);
                        let end_in_bits =
                            destination_offset_in_bits + len_in_bits;
                        assert_eq!(
                            progress.destination_position_in_bits,
                            end_in_bits
                        );
                        let original = sentinel_text.repeat(64);
                        let expected = std::format!(
                            "{}{}{}",
                            &original[..destination_offset_in_bits],
                            &LOGICAL_BITS[..len_in_bits],
                            &original[end_in_bits..]
                        );
                        assert_block_text(&block, &expected);
                    }
                }
            }
        }
    }

    #[test]
    fn empty_spans_preserve_storage() {
        for (input, offset_in_bits) in
            [(&[][..], 0), (&[0xff][..], 0), (&[0xff][..], 8)]
        {
            let source = BitSpan::new(input, offset_in_bits, 0).unwrap();
            for destination_offset_in_bits in 0..=512 {
                let mut block = [0xa5; 64];
                let progress =
                    pack_bits(source, &mut block, destination_offset_in_bits)
                        .unwrap();
                assert_eq!(progress.consumed_in_bits, 0);
                assert_eq!(
                    progress.destination_position_in_bits,
                    destination_offset_in_bits
                );
                assert_eq!(block, [0xa5; 64]);
            }
        }
    }

    #[test]
    fn full_block_and_partial_copies() {
        // 520 ones ensure even position zero must stop at the block end.
        let input = [0xff; 65];
        let source = BitSpan::new(&input, 0, 520).unwrap();
        for destination_offset_in_bits in [0, 447, 448, 511, 512] {
            let mut block = [0xa5; 64];
            let progress =
                pack_bits(source, &mut block, destination_offset_in_bits)
                    .unwrap();
            assert_eq!(
                progress.consumed_in_bits,
                512 - destination_offset_in_bits
            );
            assert_eq!(progress.destination_position_in_bits, 512);
            let original = "10100101".repeat(64);
            let expected = std::format!(
                "{}{}",
                &original[..destination_offset_in_bits],
                "1".repeat(512 - destination_offset_in_bits)
            );
            assert_block_text(&block, &expected);
            // A second invocation at the returned full position cannot write.
            let full_block = block;
            assert_eq!(
                pack_bits(source, &mut block, 512).unwrap().consumed_in_bits,
                0
            );
            assert_eq!(block, full_block);
        }
    }

    #[test]
    fn rejected_operations_preserve_storage() {
        use PackingError::*;
        let input = [0x80];
        for (offset_in_bits, len_in_bits, error) in [
            (0, 9, SourceOutOfRange),
            (8, 1, SourceOutOfRange),
            (9, 0, SourceOutOfRange),
            (usize::MAX, 0, SourceOutOfRange),
            (usize::MAX, 1, ArithmeticOverflow),
            (1, usize::MAX, ArithmeticOverflow),
        ] {
            for destination_offset_in_bits in [0, 511, 512] {
                let mut block = [0x5a; 64];
                let result = BitSpan::new(&input, offset_in_bits, len_in_bits)
                    .and_then(|source| {
                        pack_bits(
                            source,
                            &mut block,
                            destination_offset_in_bits,
                        )
                    });
                assert_eq!(result, Err(error));
                assert_eq!(block, [0x5a; 64]);
            }
        }
        assert_eq!(BitSpan::new(&[], 0, 1), Err(SourceOutOfRange));
        assert_eq!(BitSpan::new(&[], 1, 0), Err(SourceOutOfRange));
        for len_in_bits in [0, 1] {
            let source = BitSpan::new(&input, 0, len_in_bits).unwrap();
            for destination_offset_in_bits in [513, usize::MAX] {
                let mut block = [0x5a; 64];
                assert_eq!(
                    pack_bits(source, &mut block, destination_offset_in_bits),
                    Err(DestinationOutOfRange)
                );
                assert_eq!(block, [0x5a; 64]);
            }
        }
        // Capacity multiplication overflow is guarded in production but
        // cannot be exercised with a real slice of that size here in safe
        // Rust. Endpoint overflow above is exercised without fake slices.
    }
}
