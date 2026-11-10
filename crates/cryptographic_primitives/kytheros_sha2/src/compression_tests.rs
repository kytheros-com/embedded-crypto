use super::*;

// FIPS 180-4 §5.3.3. The primitive accepts arbitrary chaining values.
const IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
    0x1f83d9ab, 0x5be0cd19,
];

const ASYMMETRIC: [u32; 8] = [
    0x01234567, 0x89abcdef, 0x13579bdf, 0x2468ace0, 0xfedcba98, 0x76543210,
    0xdeadbeef, 0x80ff007f,
];

// Direct FIPS schedule and array shifts for the working-state update.
// This shares K with production, so differential agreement cannot verify
// the constants independently. The published abc digest checks the combined
// effect of the constants, schedule, rounds and feed-forward for one block.
fn reference(
    initial: [u32; 8],
    block: &[u8; 64],
    mut observe: impl FnMut(usize, u32, [u32; 8]),
) -> [u32; 8] {
    let mut w = [0_u32; 64];
    for (word, bytes) in w.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = bytes
            .iter()
            .fold(0, |acc, &byte| (acc << 8) | u32::from(byte));
    }
    for t in 16..64 {
        let x = w[t - 15];
        let y = w[t - 2];
        let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
        let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
        w[t] = w[t - 16]
            .wrapping_add(s0)
            .wrapping_add(w[t - 7])
            .wrapping_add(s1);
    }

    let mut v = initial;
    for t in 0..64 {
        let a = v[0];
        let e = v[4];
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = v[6] ^ (e & (v[5] ^ v[6]));
        let majority = (a & v[1]) | ((a | v[1]) & v[2]);
        let delta = w[t]
            .wrapping_add(K[t])
            .wrapping_add(choose)
            .wrapping_add(s1)
            .wrapping_add(v[7]);
        v.copy_within(0..7, 1);
        v[4] = v[4].wrapping_add(delta);
        v[0] = delta.wrapping_add(s0).wrapping_add(majority);
        observe(t, w[t], v);
    }

    core::array::from_fn(|i| initial[i].wrapping_add(v[i]))
}

fn compare(block: [u8; 64]) {
    for initial in [IV, [0; 8], [u32::MAX; 8], ASYMMETRIC] {
        let mut actual = initial;
        compress(&mut actual, &block, &mut [0xdeadbeef; 16]);
        assert_eq!(
            actual,
            reference(initial, &block, |_, _, _| {}),
            "initial={initial:08x?}, block={block:02x?}"
        );
    }
}

#[test]
fn structured_and_single_bit_blocks() {
    for byte in [0, 0xff, 0xaa, 0x55] {
        compare([byte; 64]);
    }
    compare(core::array::from_fn(|i| i as u8));
    compare(core::array::from_fn(|i| {
        (i as u8).wrapping_mul(37).wrapping_add(0x81)
    }));
    for bit in [0, 7, 8, 31, 32, 127, 255, 479, 480, 511] {
        let mut block = [0; 64];
        block[bit / 8] = 0x80 >> (bit % 8);
        compare(block);
    }
}

#[test]
fn deterministic_pseudorandom_blocks() {
    // xorshift32 with shifts (13, 17, 5). Take the low byte after each
    // transition from the fixed nonzero seed for reproducible sampling.
    let mut x = 0x243f6a88_u32;
    for _ in 0..64 {
        compare(core::array::from_fn(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        }));
    }
}

#[test]
fn abc_published_known_answer() {
    // FIPS 180-4 §5.1.1 layout for exactly 24 message bits.
    let mut block = [0; 64];
    block[..4].copy_from_slice(b"abc\x80");
    block[63] = 24;
    let mut state = IV;
    compress(&mut state, &block, &mut [0; 16]);
    let mut digest = [0; 32];
    for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }

    // NIST NSRL Test Data, SHA-256 example for the ASCII string "abc".
    // NIST NSRL Test Data is linked from the SHA-256 module documentation.
    assert_eq!(
        digest,
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40,
            0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17,
            0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ]
    );
}

#[test]
fn round_checkpoints_and_feed_forward() {
    let block = core::array::from_fn(|i| i as u8);
    let mut schedule_workspace = [0; 16];
    let mut schedule = MessageSchedule::new(&block, &mut schedule_workspace);
    let mut working = ASYMMETRIC;
    let expected = reference(ASYMMETRIC, &block, |t, word, after_round| {
        let actual_word = schedule.next_word().unwrap();
        assert_eq!(actual_word, word, "schedule word {t}");
        working = round(working, K[t], actual_word);
        if matches!(t, 0 | 1 | 15 | 16 | 31 | 32 | 63) {
            assert_eq!(working, after_round, "after round {t}");
        }
    });

    let mut actual = ASYMMETRIC;
    compress(&mut actual, &block, &mut schedule_workspace);
    for (i, word) in actual.into_iter().enumerate() {
        assert_eq!(word, ASYMMETRIC[i].wrapping_add(working[i]), "H_{i}");
    }
    assert_eq!(actual, expected);
}

#[test]
fn sequential_blocks_reuse_workspace() {
    let mut workspace = [u32::MAX; 16];
    let mut actual = ASYMMETRIC;
    let mut expected = ASYMMETRIC;
    let mut clean_workspace = [0; 16];
    let mut fresh = ASYMMETRIC;
    let sequential = core::array::from_fn(|i| i as u8);
    for input in [sequential, [0xff; 64], sequential] {
        let block = input;
        compress(&mut actual, &block, &mut workspace);
        clean_workspace.fill(0);
        compress(&mut fresh, &block, &mut clean_workspace);
        expected = reference(expected, &input, |_, _, _| {});
        assert_eq!(block, input);
        assert_eq!(actual, expected);
        assert_eq!(fresh, actual);
    }
}
