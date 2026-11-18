//! Functional tests for FIPS 180-4 §4.1.3. Expected values are derived
//! locally from Boolean truth tables and bit positions. The Boolean tests
//! cover each single-bit triple at every position. Word identities and
//! linearity are sampled, with no claim of exhaustive word coverage.

use super::*;

const EDGE_WORDS: [u64; 8] = [
    0,
    u64::MAX,
    0xaaaa_aaaa_aaaa_aaaa,
    0x5555_5555_5555_5555,
    0xffff_ffff_0000_0000,
    0x0000_0000_ffff_ffff,
    0x0123_4567_89ab_cdef,
    0x8000_0001_8000_0001,
];

#[test]
fn boolean_truth_tables() {
    // Rows are ordered by the binary value of xyz, with x first.
    let ch_expected = [0, 1, 0, 1, 0, 0, 1, 1];
    let maj_expected = [0, 0, 0, 1, 0, 1, 1, 1];

    for position in 0..u64::BITS {
        for row in 0..8 {
            let x = ((row as u64 >> 2) & 1) << position;
            let y = ((row as u64 >> 1) & 1) << position;
            let z = (row as u64 & 1) << position;
            assert_eq!(
                ch(x, y, z),
                ch_expected[row] << position,
                "Ch row {row}, bit {position}"
            );
            assert_eq!(
                maj(x, y, z),
                maj_expected[row] << position,
                "Maj row {row}, bit {position}"
            );
        }
    }
}

#[test]
fn ch_identities() {
    for x in EDGE_WORDS {
        for y in EDGE_WORDS {
            assert_eq!(ch(x, y, y), y);
            assert_eq!(ch(x, u64::MAX, 0), x);
            for z in EDGE_WORDS {
                assert_eq!(ch(x, y, z), ch(!x, z, y));
            }
        }
    }
}

#[test]
fn maj_identities() {
    for x in EDGE_WORDS {
        for y in EDGE_WORDS {
            assert_eq!(maj(x, x, y), x);
            assert_eq!(maj(x, !x, y), y);
            for z in EDGE_WORDS {
                // These two permutations generate all six permutations.
                assert_eq!(maj(x, y, z), maj(y, x, z));
                assert_eq!(maj(x, y, z), maj(y, z, x));
            }
        }
    }
}

// Work backward from each output position, counting contributing one bits.
// For a rotation, the source index wraps modulo the word width. For a
// shift, an index past the most significant bit contributes zero. The
// parity of the count sets the output bit. Indices start at the LSB.
fn sigma_oracle(
    x: u64,
    rotations_in_bits: &[u32],
    shift_in_bits: Option<u32>,
) -> u64 {
    let mut expected = 0;
    for output_bit in 0..u64::BITS {
        let mut ones = 0;
        for rotation_in_bits in rotations_in_bits {
            let source_bit = (output_bit + rotation_in_bits) % u64::BITS;
            ones += (x >> source_bit) & 1;
        }
        if let Some(shift_in_bits) = shift_in_bits {
            let source_bit = output_bit + shift_in_bits;
            if source_bit < u64::BITS {
                ones += (x >> source_bit) & 1;
            }
        }
        expected |= (ones % 2) << output_bit;
    }
    expected
}

fn check_sigma(
    sigma: fn(u64) -> u64,
    rotations_in_bits: &[u32],
    shift_in_bits: Option<u32>,
) {
    // The basis spans both 32-bit halves and covers every wrap and
    // zero-fill position. Basis agreement relies on linearity to extend
    // to arbitrary words. The checks below sample that property.
    for position in 0..u64::BITS {
        let x = 1 << position;
        assert_eq!(
            sigma(x),
            sigma_oracle(x, rotations_in_bits, shift_in_bits),
            "one-hot input at bit {position}"
        );
    }

    for x in EDGE_WORDS {
        assert_eq!(
            sigma(x),
            sigma_oracle(x, rotations_in_bits, shift_in_bits),
            "edge word {x:#018x}"
        );
        for y in EDGE_WORDS {
            assert_eq!(
                sigma(x ^ y),
                sigma(x) ^ sigma(y),
                "linearity for {x:#018x} and {y:#018x}"
            );
        }
    }
}

#[test]
fn big_sigma_0_test() {
    check_sigma(big_sigma_0, &[28, 34, 39], None);
}

#[test]
fn big_sigma_1_test() {
    check_sigma(big_sigma_1, &[14, 18, 41], None);
}

#[test]
fn small_sigma_0_test() {
    check_sigma(small_sigma_0, &[1, 8], Some(7));
}

#[test]
fn small_sigma_1_test() {
    check_sigma(small_sigma_1, &[19, 61], Some(6));
}
