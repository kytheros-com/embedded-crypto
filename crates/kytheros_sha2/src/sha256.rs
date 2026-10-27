//! SHA-256 word functions from NIST FIPS 180-4 (August 2015), §4.1.2.
//!
//! Words are 32-bit unsigned integers. Following §3.1, the most significant
//! bit is the leftmost bit of the represented bit string; future byte decoding
//! must preserve NIST's big-endian convention. These functions operate on word
//! values independently of host byte order and perform no message parsing.
//!
//! Inputs and outputs are temporary working values. No state is retained here;
//! the MCL owns authoritative cryptographic state and any eventual commits.
//! This module implements only logical functions, not a complete hash.
//!
//! Specification: <https://doi.org/10.6028/NIST.FIPS.180-4>.

// 4.1.2 SHA-224 and SHA-256 Functions (32-bit words).

/// `Ch(x, y, z) = (x AND y) XOR (NOT x AND z)` (§4.1.2, Eq. 4.2).
#[inline]
pub const fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// `Maj(x, y, z) = (x AND y) XOR (x AND z) XOR (y AND z)` (Eq. 4.3).
#[inline]
pub const fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// `Σ₀(x) = ROTR²(x) XOR ROTR¹³(x) XOR ROTR²²(x)` (Eq. 4.4).
#[inline]
pub const fn big_sigma_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// `Σ₁(x) = ROTR⁶(x) XOR ROTR¹¹(x) XOR ROTR²⁵(x)` (Eq. 4.5).
#[inline]
pub const fn big_sigma_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// `σ₀(x) = ROTR⁷(x) XOR ROTR¹⁸(x) XOR SHR³(x)` (Eq. 4.6).
#[inline]
pub const fn small_sigma_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// `σ₁(x) = ROTR¹⁷(x) XOR ROTR¹⁹(x) XOR SHR¹⁰(x)` (Eq. 4.7).
#[inline]
pub const fn small_sigma_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORD_SIZE_IN_BITS: u32 = 32;
    const TEST_X: u32 = 0x0000f000;
    const TEST_Y: u32 = 0x00033000;
    const TEST_Z: u32 = 0x00055000;

    const CH_ANSWERS: u32 = 0x00053000;
    const MAJ_ANSWERS: u32 = 0x00017000;

    const SIGMA_BULK_TEST_NUM_ITERATIONS: u32 = 50_000;

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

    const LFSR_MASK: u32 = 0x8020_0003;

    #[inline]
    fn lfsr_next(x: u32) -> u32 {
        (x >> 1) ^ (0_u32.wrapping_sub(x & 1) & LFSR_MASK)
    }

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

    fn apply_matrix(x: u32, a: [u32; 32]) -> u32 {
        let mut results: u32 = 0;
        for i in 0..32 {
            results ^= ((((x << (31 - i)) as i32) >> 31) as u32) & a[i];
        }
        results
    }

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
