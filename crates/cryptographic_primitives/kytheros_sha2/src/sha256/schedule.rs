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
#[path = "../../tests/sha256_tests/unit_tests/schedule_unit_tests.rs"]
mod tests;
