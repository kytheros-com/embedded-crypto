//! SHA-256 message schedule, FIPS 180-4 (August 2015), §6.2.2 step 1.
//!
//! For 16 <= t < 64, §6.2.2 step 1 defines
//! `W_t = sigma_1(W_(t-2)) + W_(t-7) + sigma_0(W_(t-15)) + W_(t-16)`,
//! with additions modulo 2^32. The first sixteen words are decoded directly
//! from the block; the type contract below gives the circular invariants.
//!
//! Tests compare every word and retained window against a direct 64-word
//! recurrence, including ring boundaries, exhaustion and workspace reuse.
//! That reference shares sigma helpers; separate local arithmetic fixtures
//! include a reproducible Python derivation and a hand-derived carry case.
//! Structured, single-bit and deterministic sampled blocks are development
//! evidence, not exhaustive or full-hash verification.
//!
//! Only the caller's sixteen `u32` words hold schedule data. A cursor
//! records the next schedule index; no block copy or 64-word schedule is
//! retained. This is temporary primitive working storage, not authoritative
//! MCL state. The caller owns storage reuse and cleanup; dropping the
//! schedule does not establish zeroization. No padding, compression rounds,
//! hash initialization, finalization, or digest generation is performed.

use super::{small_sigma_0, small_sigma_1};

/// Sequential access to W_0 through W_63 using caller-owned working words.
///
/// Initially `q[j] = W_j = M_j` for `0 <= j < 16`. Before producing W_t
/// for `16 <= t < 64`, `q[k % 16] = W_k` for `t - 16 <= k < t`.
/// Thus the four recurrence operands occupy distinct slots, and the slot
/// `t % 16` still holds W_(t-16). After replacing that slot with W_t,
/// the retained window is `t - 15 <= k <= t`. For `t < 16`, reading the
/// initial words leaves the array unchanged.
///
/// These ring indices are a Kytheros storage choice for the direct FIPS
/// 180-4 §6.2.2 step 1 recurrence. Word arithmetic is modulo 2^32 (§3.2);
/// the small-sigma functions follow §4.1.2, equations (4.6) and (4.7).
/// Private fields and exclusive borrowing prevent caller mutation of the
/// window or cursor while the schedule is active. The cursor stays 0..=64.
pub(super) struct MessageSchedule<'a> {
    q: &'a mut [u32; 16],
    next_word_index: usize,
}

impl<'a> MessageSchedule<'a> {
    /// Decodes a complete block into all sixteen caller-provided words.
    ///
    /// Byte positions `4*j..4*j+4` encode M_j in big-endian order:
    /// `b[4*j]*2^24 + b[4*j+1]*2^16 + b[4*j+2]*2^8 + b[4*j+3]`.
    /// This preserves the MSb-first convention of FIPS 180-4 §3.1 and
    /// block-word ordering of §5.2.1, independently of native endianness.
    /// Every initial value in `q` is overwritten. The block is not retained.
    /// Exact array types enforce the storage sizes; all byte indices are
    /// bounded by 63. Construction cannot fail or panic for any block.
    pub(super) fn new(block: &[u8; 64], q: &'a mut [u32; 16]) -> Self {
        for (word_index, word) in q.iter_mut().enumerate() {
            let offset_in_bytes = 4 * word_index;
            *word = u32::from_be_bytes([
                block[offset_in_bytes],
                block[offset_in_bytes + 1],
                block[offset_in_bytes + 2],
                block[offset_in_bytes + 3],
            ]);
        }
        Self {
            q,
            next_word_index: 0,
        }
    }

    /// Returns the next word, or `None` after exactly 64 successful calls.
    ///
    /// Exhaustion leaves storage and cursor unchanged on every later call.
    /// All input words are supported without failure or panic. Subtractions
    /// occur only for `t >= 16`; ring indices are 0..16 and the cursor never
    /// exceeds 64. Each call uses scalar temporaries and no heap allocation.
    pub(super) fn next_word(&mut self) -> Option<u32> {
        let t = self.next_word_index;
        if t == 64 {
            return None;
        }
        let slot = t % 16;
        if t >= 16 {
            // Evaluate every operand, including W_(t-16), before writing.
            let word = small_sigma_1(self.q[(t - 2) % 16])
                .wrapping_add(self.q[(t - 7) % 16])
                .wrapping_add(small_sigma_0(self.q[(t - 15) % 16]))
                .wrapping_add(self.q[slot]);
            self.q[slot] = word;
        }
        let word = self.q[slot];
        self.next_word_index += 1;
        Some(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Direct FIPS §6.2.2 step 1 oracle: ordinary indices and 64 words.
    // Decode independently using positional shifts, not from_be_bytes.
    // This reference never calls MessageSchedule. It shares the existing
    // sigma helpers, so agreement cannot independently establish their
    // correctness. Fixed arithmetic fixtures below provide a separate check.
    fn reference_schedule(block: &[u8; 64]) -> [u32; 64] {
        let mut w = [0_u32; 64];
        for (word_index, word) in w[..16].iter_mut().enumerate() {
            let offset_in_bytes = 4 * word_index;
            *word = (u32::from(block[offset_in_bytes]) << 24)
                | (u32::from(block[offset_in_bytes + 1]) << 16)
                | (u32::from(block[offset_in_bytes + 2]) << 8)
                | u32::from(block[offset_in_bytes + 3]);
        }
        for t in 16..64 {
            w[t] = small_sigma_1(w[t - 2])
                .wrapping_add(w[t - 7])
                .wrapping_add(small_sigma_0(w[t - 15]))
                .wrapping_add(w[t - 16]);
        }
        w
    }

    // Compare every output and the entire retained window after each word.
    // These are deterministic functional samples, not exhaustive proofs,
    // full-hash tests, algorithm validation, or embedded execution evidence.
    fn compare_schedule(block: &[u8; 64]) {
        let expected = reference_schedule(block);
        let mut q = [0xdead_beef; 16];
        let mut schedule = MessageSchedule::new(block, &mut q);
        assert_eq!(*schedule.q, expected[..16]);
        for (t, &word) in expected.iter().enumerate() {
            assert_eq!(schedule.next_word(), Some(word), "W_{t}");
            if t >= 15 {
                for (k, &retained_word) in
                    expected.iter().enumerate().take(t + 1).skip(t - 15)
                {
                    assert_eq!(
                        schedule.q[k % 16],
                        retained_word,
                        "t={t}, k={k}"
                    );
                }
            }
        }
        let final_window = *schedule.q;
        for _ in 0..3 {
            assert_eq!(schedule.next_word(), None);
            assert_eq!(schedule.next_word_index, 64);
            assert_eq!(*schedule.q, final_window);
        }
    }

    fn sequential_block() -> [u8; 64] {
        core::array::from_fn(|byte_index| byte_index as u8)
    }

    #[test]
    fn structured_blocks_match_reference() {
        for byte in [0x00, 0xff, 0xaa, 0x55] {
            compare_schedule(&[byte; 64]);
        }
        compare_schedule(&sequential_block());
        // Deliberately asymmetric bytes across both byte and word positions.
        let asymmetric = core::array::from_fn(|byte_index| {
            (byte_index as u8).wrapping_mul(37).wrapping_add(0x81)
        });
        compare_schedule(&asymmetric);
    }

    #[test]
    fn representative_single_bits_match_reference() {
        // MSb-first positions: block endpoints, byte/word boundaries,
        // middle words, and the recurrence's last initial operand.
        for position_in_bits in [0, 7, 8, 31, 32, 255, 256, 447, 448, 511] {
            let mut block = [0; 64];
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
            let mut block = [0; 64];
            for byte in &mut block {
                state =
                    state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
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
        for boundary in [15, 16, 31, 32, 47, 63] {
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
    fn initial_and_expanded_arithmetic_fixtures() {
        // Local constants for bytes 00..3f, not NIST-published vectors.
        // Reproduce with standalone Python 3 using integer division/modulo
        // for rotations and arbitrary-precision sums, independent of Rust:
        //
        // def r(x, n):
        //     return x//2**n + (x%2**n)*2**(32-n)
        // w = [sum((4*t+j)*256**(3-j) for j in range(4))
        //      for t in range(16)]
        // for t in range(16, 64):
        //     x, y = w[t-15], w[t-2]
        //     a = r(x, 7) ^ r(x, 18) ^ (x//8)
        //     b = r(y, 17) ^ r(y, 19) ^ (y//1024)
        //     w.append((b + w[t-7] + a + w[t-16]) % 2**32)
        //
        // The first three constants also directly encode four consecutive
        // bytes with the first byte most significant. This script is an
        // independent calculation, not independent human or formal review.
        let fixtures = [
            (0, 0x0001_0203),
            (1, 0x0405_0607),
            (15, 0x3c3d_3e3f),
            (16, 0x2d84_894c),
            (17, 0xba99_985f),
            (31, 0xb7ff_4640),
            (32, 0x528a_d2af),
            (47, 0xd8ad_8d21),
            (63, 0x24a9_7715),
        ];
        let block = sequential_block();
        let reference = reference_schedule(&block);
        let mut q = [0; 16];
        let mut schedule = MessageSchedule::new(&block, &mut q);
        for (t, &reference_word) in reference.iter().enumerate() {
            let actual = schedule.next_word().unwrap();
            if let Some(&(_, expected)) =
                fixtures.iter().find(|&&(i, _)| i == t)
            {
                assert_eq!(actual, expected, "W_{t}");
                assert_eq!(reference_word, expected, "reference W_{t}");
            }
        }
    }

    #[test]
    fn wrapping_sum_fixture_and_workspace_reuse() {
        // For x = ffffffff, sigma_1(x) = 003fffff and
        // sigma_0(x) = 1fffffff: the two all-one rotations cancel.
        // W_16 = W_17 = (003fffff + ffffffff + 1fffffff + ffffffff)
        // mod 2^32 = 203ffffc, a hand-derived carry-discarding fixture.
        let mut q = [0; 16];
        {
            let mut schedule = MessageSchedule::new(&[0xff; 64], &mut q);
            for _ in 0..16 {
                assert_eq!(schedule.next_word(), Some(u32::MAX));
            }
            assert_eq!(schedule.next_word(), Some(0x203f_fffc));
            assert_eq!(schedule.next_word(), Some(0x203f_fffc));
        }
        // A new block must replace all slots, even after early abandonment.
        let mut schedule = MessageSchedule::new(&[0; 64], &mut q);
        assert_eq!(*schedule.q, [0; 16]);
        for _ in 0..64 {
            assert_eq!(schedule.next_word(), Some(0));
        }
        assert_eq!(schedule.next_word(), None);
    }
}
