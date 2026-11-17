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

// Permutes bit positions by reversing the five-bit binary representation
// of each position's index (radix-2 butterfly bit-reversal permutation).
// This is distinct from reversing the 32-bit word's bit order.
// Applied to every operand and answer, any common bit permutation
// preserves Ch/Maj because they act independently at each position.
#[inline]
fn butterfly_permutation(x: u32) -> u32 {
    let y1 = (((x << 15) & 0x5555_0000) | (x & 0xaaaa_0000))
        | (((x >> 15) & 0x0000_aaaa) | (x & 0x0000_5555));
    (((y1 << 6) & 0x3300_3300) | (y1 & 0xcc_00_cc_00))
        | (((y1 >> 6) & 0x00_cc_00_cc) | (y1 & 0x00_33_00_33))
}

#[test]
fn butterfly_permutation_positions() {
    for i in 0_u32..32 {
        let expected = 1_u32 << (i.reverse_bits() >> 27);
        assert_eq!(butterfly_permutation(1_u32 << i), expected);
    }
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
