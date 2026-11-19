//! Differential and known-answer tests for one complete SHA-512 block.
//!
//! The reference decodes bytes by shifts, expands a flat 80-word schedule
//! and uses direct rotations and alternative Boolean formulas. It shares
//! the production constants, whose separate verification is documented in
//! `verify_round_constants.py`. The published digest is a separate oracle.

use super::{compress, round};
use crate::sha512::{
    constants::SHA_512_ROUND_CONSTANTS, schedule::MessageSchedule,
};

// FIPS 180-4 (August 2015), §5.3.5, in H_0 through H_7 order.
const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];
const ASYMMETRIC: [u64; 8] = [
    0x0123456789abcdef,
    0xfedcba9876543210,
    0x1020304050607080,
    0x8070605040302010,
    0x8000000000000001,
    0x00000000ffffffff,
    0xffffffff00000000,
    0x13579bdf02468ace,
];
const STATES: [[u64; 8]; 4] = [IV, [0; 8], [u64::MAX; 8], ASYMMETRIC];
const SEED: u64 = 0x243f6a8885a308d3;

fn reference_schedule(block: &[u8; 128]) -> [u64; 80] {
    let mut words = [0; 80];
    let (chunks, _) = block.as_chunks::<8>();
    for (word, bytes) in words.iter_mut().zip(chunks) {
        *word = bytes
            .iter()
            .fold(0, |acc, &byte| (acc << 8) | u64::from(byte));
    }
    for t in 16..80 {
        let x = words[t - 15];
        let y = words[t - 2];
        let sigma0 = x.rotate_right(1) ^ x.rotate_right(8) ^ (x >> 7);
        let sigma1 = y.rotate_right(19) ^ y.rotate_right(61) ^ (y >> 6);
        words[t] = words[t - 16]
            .wrapping_add(sigma0)
            .wrapping_add(words[t - 7])
            .wrapping_add(sigma1);
    }
    words
}

fn reference_round(working: &mut [u64; 8], constant: u64, word: u64) {
    let a = working[0];
    let e = working[4];
    let sigma0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
    let sigma1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
    let choose = working[6] ^ (e & (working[5] ^ working[6]));
    let majority = (working[1] & working[2]) | (a & (working[1] | working[2]));
    let t1 = word
        .wrapping_add(constant)
        .wrapping_add(choose)
        .wrapping_add(sigma1)
        .wrapping_add(working[7]);
    let t2 = majority.wrapping_add(sigma0);

    working.copy_within(..7, 1);
    working[4] = working[4].wrapping_add(t1);
    working[0] = t2.wrapping_add(t1);
}

fn reference_working(mut working: [u64; 8], words: [u64; 80]) -> [u64; 8] {
    for (word, constant) in words.into_iter().zip(SHA_512_ROUND_CONSTANTS) {
        reference_round(&mut working, constant, word);
    }
    working
}

fn reference(initial: [u64; 8], words: [u64; 80]) -> [u64; 8] {
    let working = reference_working(initial, words);
    core::array::from_fn(|i| initial[i].wrapping_add(working[i]))
}

fn compare_block(block: &[u8; 128], case: usize) {
    let words = reference_schedule(block);
    for (state_index, initial) in STATES.into_iter().enumerate() {
        let mut actual = initial;
        compress(&mut actual, block, &mut [0; 16]);
        assert_eq!(
            actual,
            reference(initial, words),
            "block case {case}, initial state {state_index}"
        );
    }
}

// xorshift64 with shifts (13, 7, 17), starting at SEED in each test.
// Block sampling takes the low byte after each transition. Round sampling
// takes the entire resulting word. These are deterministic samples.
fn next_sample(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn asymmetric_block() -> [u8; 128] {
    core::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(0x81))
}

#[test]
fn structured_blocks() {
    let alternating = core::array::from_fn(|i| [0xaa, 0x55][i % 2]);
    let sequential = core::array::from_fn(|i| i as u8);
    for (case, block) in [
        [0; 128],
        [0xff; 128],
        alternating,
        sequential,
        asymmetric_block(),
    ]
    .iter()
    .enumerate()
    {
        compare_block(block, case);
    }
}

#[test]
fn all_single_bit_blocks() {
    for bit_index in 0..1024 {
        let mut block = [0; 128];
        block[bit_index / 8] = 0x80 >> (bit_index % 8);
        compare_block(&block, bit_index);
    }
}

#[test]
fn deterministic_pseudorandom_blocks() {
    let mut seed = SEED;
    for case in 0..128 {
        let block = core::array::from_fn(|_| next_sample(&mut seed) as u8);
        compare_block(&block, case);
    }
}

#[test]
fn individual_rounds() {
    let boundaries = [
        0,
        u64::MAX,
        0xaaaaaaaaaaaaaaaa,
        0x5555555555555555,
        1 << 63,
        1,
        (1 << 32) - 1,
        1 << 32,
        (1 << 32) + 1,
        u64::MAX - 1,
    ];
    for (index, constant) in boundaries.into_iter().enumerate() {
        for word in boundaries {
            let working = core::array::from_fn(|i| {
                boundaries[(index + i) % boundaries.len()]
            });
            let mut expected = working;
            reference_round(&mut expected, constant, word);
            assert_eq!(round(working, constant, word), expected);
        }
    }

    let mut seed = SEED;
    for case in 0..128 {
        let working = core::array::from_fn(|_| next_sample(&mut seed));
        let constant = next_sample(&mut seed);
        let word = next_sample(&mut seed);
        let mut expected = working;
        reference_round(&mut expected, constant, word);
        assert_eq!(round(working, constant, word), expected, "sample {case}");
    }
}

#[test]
fn round_carry_and_overflow() {
    // With all working words zero, only K_t + W_t contributes to T1.
    // The two changed working words must both contain that modular sum.
    for (constant, word, sum) in [
        (0x00000000ffffffff, 1, 0x0000000100000000),
        (u64::MAX, 2, 1),
    ] {
        let expected = [sum, 0, 0, 0, sum, 0, 0, 0];
        assert_eq!(round([0; 8], constant, word), expected);
    }
}

#[test]
fn round_checkpoints() {
    let block = asymmetric_block();
    let mut workspace = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut workspace);
    let mut actual = ASYMMETRIC;
    let mut expected = ASYMMETRIC;
    for (t, (expected_word, constant)) in reference_schedule(&block)
        .into_iter()
        .zip(SHA_512_ROUND_CONSTANTS)
        .enumerate()
    {
        let word = schedule.next_word().unwrap();
        actual = round(actual, constant, word);
        reference_round(&mut expected, constant, expected_word);
        if matches!(t, 0 | 1 | 15 | 16 | 31 | 32 | 47 | 48 | 63 | 64 | 78 | 79)
        {
            assert_eq!(word, expected_word, "schedule word {t}");
            assert_eq!(actual, expected, "working state after round {t}");
        }
    }
    assert_eq!(schedule.next_word(), None);
}

#[test]
fn feed_forward_preserves_original_words_and_wraps() {
    let block = asymmetric_block();
    let initial = [u64::MAX; 8];
    let working = reference_working(initial, reference_schedule(&block));
    let mut actual = initial;
    compress(&mut actual, &block, &mut [0; 16]);
    for (i, word) in working.into_iter().enumerate() {
        // Confirm this fixture forces both carries for every state word.
        assert!((u32::MAX).overflowing_add(word as u32).1);
        let (sum, overflowed) = initial[i].overflowing_add(word);
        assert!(overflowed, "overflow in H_{i}");
        assert_eq!(actual[i], sum, "feed-forward H_{i}");
    }
    assert_ne!(actual, working);
}

#[test]
fn chained_blocks_reuse_workspace() {
    let mut reused = [u64::MAX; 16];
    let mut actual = ASYMMETRIC;
    let mut fresh = ASYMMETRIC;
    let mut expected = ASYMMETRIC;
    for input in [
        asymmetric_block(),
        [0xff; 128],
        [0; 128],
        asymmetric_block(),
    ] {
        let block = input;
        let words = reference_schedule(&input);
        // Inspect all initial slots using a separate copy of the dirty
        // workspace. Then run compression with the untouched dirty array.
        let mut initial_slots = reused;
        MessageSchedule::new(&block, &mut initial_slots);
        assert_eq!(initial_slots, words[..16]);

        compress(&mut actual, &block, &mut reused);
        compress(&mut fresh, &block, &mut [0; 16]);
        expected = reference(expected, words);
        assert_eq!(actual, expected);
        assert_eq!(fresh, expected);
        assert_eq!(reused, words[64..]);
        assert_eq!(block, input);
    }
}

#[test]
fn abc_published_known_answer() {
    // FIPS 180-4 §5.1.2, with the length encoded in the final 128 bits.
    let mut block = [0; 128];
    block[..4].copy_from_slice(b"abc\x80");
    block[112..].copy_from_slice(&24_u128.to_be_bytes());
    let mut state = IV;
    compress(&mut state, &block, &mut [0; 16]);

    let mut digest = [0; 64];
    let (chunks, _) = digest.as_chunks_mut::<8>();
    for (bytes, word) in chunks.iter_mut().zip(state) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }
    // NIST SHA-512 example, "abc" one-block sample, final digest.
    // See the source link in the compression module documentation.
    assert_eq!(
        digest,
        [
            0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73,
            0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9,
            0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a, 0x21,
            0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, 0x23,
            0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8,
            0x0e, 0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
        ]
    );
}
