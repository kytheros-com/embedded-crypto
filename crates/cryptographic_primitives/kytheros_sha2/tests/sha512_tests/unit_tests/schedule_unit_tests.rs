//! Development tests for FIPS 180-4 §5.2.2 and §6.4.2 step 1.
//!
//! Host tests cover block decoding, the schedule recurrence, ring updates,
//! boundary transitions, and workspace lifecycle. Basis blocks exhaust
//! single-bit inputs. Structured and generated blocks sample other inputs.
//! Embedded execution, 32-bit instruction lowering, and physical
//! side-channel behavior require separate evidence.

use super::*;

// Direct 80-word reference with independent byte decoding and a wide sum.
// It shares the previously tested sigma helpers with MessageSchedule.
// Agreement alone cannot verify sigma correctness.
fn reference_schedule(block: &[u8; 128]) -> [u64; 80] {
    let mut w = [0; 80];
    let (message_words, _) = block.as_chunks::<8>();
    for (word, bytes) in w[..16].iter_mut().zip(message_words) {
        for &byte in bytes {
            *word = (*word << 8) | u64::from(byte);
        }
    }
    for t in 16..80 {
        let sum = u128::from(small_sigma_1(w[t - 2]))
            + u128::from(w[t - 7])
            + u128::from(small_sigma_0(w[t - 15]))
            + u128::from(w[t - 16]);
        w[t] = sum as u64;
    }
    w
}

// Check every returned word, cursor advance, and retained slot.
fn compare_schedule(block: &[u8; 128]) {
    let expected = reference_schedule(block);
    let mut q = [0xdead_beef_dead_beef; 16];
    {
        let mut schedule = MessageSchedule::new(block, &mut q);
        assert_eq!(schedule.next_word_index, 0);
        assert_eq!(*schedule.q, expected[..16]);
        for (t, &word) in expected.iter().enumerate() {
            assert_eq!(schedule.next_word(), Some(word), "W_{t}");
            assert_eq!(schedule.next_word_index, t + 1);
            let start = (t + 1).saturating_sub(16);
            for (offset, &retained_word) in
                expected[start..start + 16].iter().enumerate()
            {
                let slot = (start + offset) % 16;
                assert_eq!(schedule.q[slot], retained_word, "t={t}, q[{slot}]");
            }
        }
        for _ in 0..3 {
            assert_eq!(schedule.next_word(), None);
            assert_eq!(schedule.next_word_index, 80);
            assert_eq!(*schedule.q, expected[64..]);
        }
    }
    assert_eq!(q, expected[64..]);
}

fn sequential_block() -> [u8; 128] {
    core::array::from_fn(|byte_index| byte_index as u8)
}

#[test]
fn structured_blocks_match_reference() {
    for byte in [0x00, 0xff, 0xaa, 0x55] {
        compare_schedule(&[byte; 128]);
    }
    compare_schedule(&sequential_block());
    // Deliberately asymmetric bytes across both byte and word positions.
    let asymmetric = core::array::from_fn(|byte_index| {
        (byte_index as u8).wrapping_mul(37).wrapping_add(0x81)
    });
    compare_schedule(&asymmetric);
}

#[test]
fn all_single_bit_blocks_match_reference() {
    // Each MSb-first position gives one of the 1,024 basis blocks.
    for position_in_bits in 0..1024 {
        let mut block = [0; 128];
        block[position_in_bits / 8] = 0x80 >> (position_in_bits % 8);
        compare_schedule(&block);
    }
}

#[test]
fn deterministic_pseudorandom_blocks_match_reference() {
    // 32-bit LCG: s_(n+1) = (1664525*s_n + 1013904223) mod 2^32.
    // Seed 0x6a09e667; emit the high byte AFTER each update, continuing
    // across 128 blocks. This is a reproducible sample generator, not
    // cryptographic randomness or a period/distribution claim.
    let mut state = 0x6a09_e667_u32;
    for _ in 0..128 {
        let mut block = [0; 128];
        for byte in &mut block {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = (state >> 24) as u8;
        }
        compare_schedule(&block);
    }
}

#[test]
fn circular_boundary_transitions() {
    let block = sequential_block();
    let expected = reference_schedule(&block);
    let mut q = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut q);
    for boundary in [15, 16, 31, 32, 47, 48, 63, 64, 79] {
        while schedule.next_word_index < boundary {
            schedule.next_word().unwrap();
        }
        let before = *schedule.q;
        assert_eq!(schedule.next_word(), Some(expected[boundary]));
        assert_eq!(schedule.next_word_index, boundary + 1);
        for (slot, &old_word) in before.iter().enumerate() {
            let expected_word = if boundary >= 16 && slot == boundary % 16 {
                expected[boundary]
            } else {
                old_word
            };
            assert_eq!(schedule.q[slot], expected_word, "W_{boundary}");
        }
    }
}

#[test]
fn sequential_block_fixtures() {
    // Locally derived values for bytes 00..7f. Reproduce in Python 3
    // with integer division for rotations and one wide modular sum:
    //
    // def r(x, n):
    //     return x//2**n + (x%2**n)*2**(64-n)
    // w = [sum((8*t+j)*256**(7-j) for j in range(8)) for t in range(16)]
    // for t in range(16, 80):
    //     x, y = w[t-15], w[t-2]
    //     a = r(x, 1) ^ r(x, 8) ^ (x//128)
    //     b = r(y, 19) ^ r(y, 61) ^ (y//64)
    //     w.append(sum((a, b, w[t-7], w[t-16])) % 2**64)
    let fixtures = [
        (0, 0x0001_0203_0405_0607),
        (1, 0x0809_0a0b_0c0d_0e0f),
        (15, 0x7879_7a7b_7c7d_7e7f),
        (16, 0xffeb_a2c6_2b98_ed57),
        (17, 0x6578_efdb_40ee_8224),
        (31, 0x5515_46eb_b4f5_460a),
        (32, 0xfec7_312b_1491_7132),
        (47, 0x91f6_5264_f488_3473),
        (48, 0xcd53_a5a9_12ab_29a6),
        (63, 0xc400_90e8_99f7_28f5),
        (64, 0xad6b_3ecd_7e17_de35),
        (79, 0x9841_46fa_adc9_e6e7),
    ];
    let block = sequential_block();
    let reference = reference_schedule(&block);
    let mut q = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut q);
    for (t, expected) in fixtures {
        while schedule.next_word_index < t {
            assert!(schedule.next_word().is_some());
        }
        assert_eq!(schedule.next_word(), Some(expected), "W_{t}");
        assert_eq!(reference[t], expected, "reference W_{t}");
    }
}

#[test]
fn cross_half_carry_fixture() {
    // W_0 = 2^32 - 1 and W_9 = 1. Both sigma inputs for W_16 are
    // zero, so adding 1 propagates a carry through all 32 low bits
    // and increments the high half. The low half becomes zero.
    let mut block = [0; 128];
    block[4..8].fill(0xff);
    block[79] = 1;
    compare_schedule(&block);

    let expected = 0x0000_0001_0000_0000;
    assert_eq!(reference_schedule(&block)[16], expected);
    let mut q = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut q);
    for _ in 0..16 {
        assert!(schedule.next_word().is_some());
    }
    assert_eq!(schedule.next_word(), Some(expected));
}

#[test]
fn four_operand_overflow_fixture() {
    // For M = 2^64 - 1, both rotations in each sigma cancel.
    // W_16 and W_17 sum M/64 + M + M/128 + M (integer division).
    // The exact sum is 0x2_05ff_ffff_ffff_fffc. Reduction discards
    // the high bits beyond bit 63 after carries across both halves.
    let block = [0xff; 128];
    let expected = 0x05ff_ffff_ffff_fffc;
    assert_eq!(reference_schedule(&block)[16..18], [expected; 2]);
    let mut q = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut q);
    for _ in 0..16 {
        assert_eq!(schedule.next_word(), Some(u64::MAX));
    }
    assert_eq!(schedule.next_word(), Some(expected));
    assert_eq!(schedule.next_word(), Some(expected));
}

#[test]
fn workspace_reuse_after_partial_and_full_consumption() {
    let mut q = [0xdead_beef_dead_beef; 16];
    {
        let mut schedule = MessageSchedule::new(&[0xff; 128], &mut q);
        assert_eq!(*schedule.q, [u64::MAX; 16]);
        for _ in 0..17 {
            assert!(schedule.next_word().is_some());
        }
    }
    {
        let mut schedule = MessageSchedule::new(&[0x55; 128], &mut q);
        assert_eq!(schedule.next_word_index, 0);
        assert_eq!(*schedule.q, [0x5555_5555_5555_5555; 16]);
        for expected in reference_schedule(&[0x55; 128]) {
            assert_eq!(schedule.next_word(), Some(expected));
        }
        assert!(schedule.q.iter().all(|&word| word != 0));
        assert_eq!(schedule.next_word(), None);
    }
    let mut schedule = MessageSchedule::new(&[0; 128], &mut q);
    assert_eq!(schedule.next_word_index, 0);
    assert_eq!(*schedule.q, [0; 16]);
    for _ in 0..80 {
        assert_eq!(schedule.next_word(), Some(0));
    }
    for _ in 0..3 {
        assert_eq!(schedule.next_word(), None);
        assert_eq!(schedule.next_word_index, 80);
        assert_eq!(*schedule.q, [0; 16]);
    }
}
