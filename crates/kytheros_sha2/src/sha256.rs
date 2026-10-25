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
    const WORDS: [u32; 8] = [
        0,
        u32::MAX,
        1,
        0x8000_0000,
        0xaaaa_aaaa,
        0x5555_5555,
        0x0123_4567,
        0x89ab_cdef,
    ];

    #[test]
    fn ch_truth_table_and_identities() {
        for x in [0, u32::MAX] {
            for y in [0, u32::MAX] {
                for z in [0, u32::MAX] {
                    assert_eq!(ch(x, y, z), if x == 0 { z } else { y });
                }
            }
        }
        for x in WORDS {
            for y in WORDS {
                for z in WORDS {
                    assert_eq!(ch(x, y, z), z ^ (x & (y ^ z)));
                }
                assert_eq!(ch(x, y, y), y);
                assert_eq!(ch(x, u32::MAX, 0), x);
                assert_eq!(ch(x, 0, u32::MAX), !x);
            }
        }
    }

    #[test]
    fn maj_truth_table_and_identities() {
        for x in [0, u32::MAX] {
            for y in [0, u32::MAX] {
                for z in [0, u32::MAX] {
                    let set_word_count = u32::from(x != 0) + u32::from(y != 0) + u32::from(z != 0);
                    assert_eq!(maj(x, y, z), if set_word_count >= 2 { u32::MAX } else { 0 });
                }
            }
        }
        for x in WORDS {
            for y in WORDS {
                for z in WORDS {
                    assert_eq!(maj(x, y, z), (x & y) | (z & (x | y)));
                    assert_eq!(maj(x, y, z), maj(z, x, y));
                }
                assert_eq!(maj(x, x, y), x);
                assert_eq!(maj(x, !x, y), y);
                assert_eq!(maj(x, y, 0), x & y);
                assert_eq!(maj(x, y, u32::MAX), x | y);
            }
        }
    }

    // Independent bit-by-bit oracle: destination positions count from the
    // least significant bit. No rotate operation or word-level shift formula
    // is used to construct the reference rotation.
    fn reference_sigma(x: u32, rotations_in_bits: &[u32], shift_in_bits: Option<u32>) -> u32 {
        let mut result = 0;
        for destination_offset_in_bits in 0..WORD_SIZE_IN_BITS {
            let mut bit = 0;
            for rotation_in_bits in rotations_in_bits {
                let source_offset_in_bits =
                    (destination_offset_in_bits + rotation_in_bits) % WORD_SIZE_IN_BITS;
                bit ^= (x >> source_offset_in_bits) & 1;
            }
            if let Some(shift_in_bits) = shift_in_bits {
                let source_offset_in_bits = destination_offset_in_bits + shift_in_bits;
                if source_offset_in_bits < WORD_SIZE_IN_BITS {
                    bit ^= (x >> source_offset_in_bits) & 1;
                }
            }
            result |= bit << destination_offset_in_bits;
        }
        result
    }

    fn check_sigma(
        function: fn(u32) -> u32,
        rotations_in_bits: &[u32],
        shift_in_bits: Option<u32>,
    ) {
        assert_eq!(function(0), 0);
        for x in WORDS {
            assert_eq!(
                function(x),
                reference_sigma(x, rotations_in_bits, shift_in_bits)
            );
            for y in WORDS {
                assert_eq!(function(x ^ y), function(x) ^ function(y));
            }
        }
        // Every basis bit exercises wraparound and logical-shift boundaries.
        for bit_offset_in_bits in 0..WORD_SIZE_IN_BITS {
            let x = 1 << bit_offset_in_bits;
            assert_eq!(
                function(x),
                reference_sigma(x, rotations_in_bits, shift_in_bits)
            );
            assert_eq!(function(!x), function(x) ^ function(u32::MAX));
        }
    }

    #[test]
    fn big_sigma_0_reference_and_identities() {
        check_sigma(big_sigma_0, &[2, 13, 22], None);
        assert_eq!(big_sigma_0(u32::MAX), u32::MAX);
    }

    #[test]
    fn big_sigma_1_reference_and_identities() {
        check_sigma(big_sigma_1, &[6, 11, 25], None);
        assert_eq!(big_sigma_1(u32::MAX), u32::MAX);
    }

    #[test]
    fn small_sigma_0_reference_and_identities() {
        check_sigma(small_sigma_0, &[7, 18], Some(3));
        assert_eq!(small_sigma_0(u32::MAX), 0x1fff_ffff);
    }

    #[test]
    fn small_sigma_1_reference_and_identities() {
        check_sigma(small_sigma_1, &[17, 19], Some(10));
        assert_eq!(small_sigma_1(u32::MAX), 0x003f_ffff);
    }
}
