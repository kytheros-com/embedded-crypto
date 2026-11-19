//! SHA-512 message schedule, FIPS 180-4 (August 2015), §6.4.2 step 1.
//!
//! For `16 <= t < 80`, the standard defines
//! `W_t = sigma_1(W_(t-2)) + W_(t-7) + sigma_0(W_(t-15)) + W_(t-16)`
//! with addition modulo 2^64. The first sixteen words come from the block.
//!
//! The caller lends sixteen temporary working words for the lifetime of
//! the schedule. The schedule retains only that borrow and a cursor, uses
//! scalar temporaries, and allocates no heap storage. The caller can reuse
//! the workspace after the borrow ends and remains responsible for cleanup.
//! Ending the borrow leaves the retained words in place.

use super::{small_sigma_0, small_sigma_1};

/// Sequential access to W_0 through W_79 using caller-owned working words.
///
/// Initially `q[j] = W_j = M_j` for `0 <= j < 16`. Before producing W_t
/// for `16 <= t < 80`, `q[k % 16] = W_k` for `t - 16 <= k < t`.
/// Thus the four recurrence operands occupy distinct slots, and the slot
/// `t % 16` still holds W_(t-16). After replacing that slot with W_t,
/// the retained window is `t - 15 <= k <= t`. For `t < 16`, reading the
/// initial words leaves the array unchanged.
///
/// These ring indices are a Kytheros storage choice for the direct FIPS
/// 180-4 §6.4.2 step 1 recurrence. Word arithmetic is modulo 2^64 (§3.2);
/// the small-sigma functions follow §4.1.3, equations (4.12) and (4.13).
/// Private fields and exclusive borrowing prevent caller mutation of the
/// window or cursor while the schedule is active. The cursor stays 0..=80.
pub(super) struct MessageSchedule<'a> {
    q: &'a mut [u64; 16],
    next_word_index: usize,
}

impl<'a> MessageSchedule<'a> {
    /// Decodes a complete block into all sixteen caller-provided words.
    ///
    /// Bytes `8*j..8*j+8` encode M_j as a numeric `u64`. The first byte
    /// supplies bits 63 through 56 and the last supplies bits 7 through 0.
    /// This preserves the MSb-first convention of FIPS 180-4 §3.1 and
    /// block-word ordering of §5.2.2, independently of native endianness.
    /// Every initial value in `q` is overwritten. The block is not retained.
    /// Exact array types enforce the storage sizes; all byte indices are
    /// bounded by 127. Construction cannot fail or panic for any block.
    pub(super) fn new(block: &[u8; 128], q: &'a mut [u64; 16]) -> Self {
        for (word_index, word) in q.iter_mut().enumerate() {
            let offset_in_bytes = 8 * word_index;
            *word = u64::from_be_bytes([
                block[offset_in_bytes],
                block[offset_in_bytes + 1],
                block[offset_in_bytes + 2],
                block[offset_in_bytes + 3],
                block[offset_in_bytes + 4],
                block[offset_in_bytes + 5],
                block[offset_in_bytes + 6],
                block[offset_in_bytes + 7],
            ]);
        }
        Self {
            q,
            next_word_index: 0,
        }
    }

    /// Returns the next word, or `None` after exactly 80 successful calls.
    ///
    /// Exhaustion leaves storage and cursor unchanged on every later call.
    /// All input words are supported without failure or panic. Subtractions
    /// occur only for `t >= 16`; ring indices are 0..16 and the cursor never
    /// exceeds 80. Each call uses scalar temporaries and no heap allocation.
    pub(super) fn next_word(&mut self) -> Option<u64> {
        let t = self.next_word_index;
        if t == 80 {
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
#[path = "../../tests/sha512_tests/unit_tests/schedule_unit_tests.rs"]
mod tests;
