//! Single-block SHA-512 compression, [FIPS 180-4 (August 2015)][fips],
//! §6.4.2.
//!
//! Step 2 initializes `(a,b,c,d,e,f,g,h)` from chaining words H_0..H_7.
//! Step 3 computes `T1 = h + Sigma_1(e) + Ch(e,f,g) + K_t + W_t` and
//! `T2 = Sigma_0(a) + Maj(a,b,c)` and updates the working words to
//! `(T1+T2,a,b,c,d+T1,e,f,g)`. Step 4 adds the final working words to
//! the original chaining words. All word additions wrap modulo 2^64
//! (§3.2), discarding overflow beyond bit 63.
//!
//! Tests cover structured and sampled inputs, every single-bit block,
//! round checkpoints, feed-forward and workspace reuse. The reference uses
//! a full schedule and separate round formulas, with the published round
//! constants shared by both paths. A manually padded block checks the
//! result against the "abc" sample in the SHA-512 PDF listed under
//! Secure Hashing in NIST Example Values, linked from the
//! [NIST Hash Functions page][examples]. These host tests exercise
//! compression with caller-prepared blocks. They provide no evidence of
//! embedded execution or algorithm or module validation.
//!
//! [fips]: https://doi.org/10.6028/NIST.FIPS.180-4
//! [examples]: https://csrc.nist.gov/Projects/Hash-Functions

use super::constants::SHA_512_ROUND_CONSTANTS;
use super::schedule::MessageSchedule;
use super::{big_sigma_0, big_sigma_1, ch, maj};

/// Compresses one complete 1024-bit block into eight chaining words.
///
/// The schedule reads consecutive big-endian words from the block (§5.2.2)
/// with the most significant bit first in each byte (§3.1). State words are
/// numeric `u64` values in H_0..H_7 order on input and output, independent
/// of host byte order. Any chaining state and block contents are valid.
///
/// The caller owns the working copy and the 16-word (128-byte) schedule
/// workspace. The MCL owns authoritative state and controls commits.
/// The schedule overwrites every workspace word before the first round.
/// The eight local working words, schedule cursor and scalar temporaries
/// require no dynamic allocation. The block is borrowed read-only, with
/// no block copy and no retained references after the call.
///
/// The input state is preserved through all 80 rounds. A fresh schedule
/// yields exactly one word per constant, so exhaustion cannot occur
/// during those rounds. Feed-forward then updates the caller's state.
/// Workspace contents remain after return. The caller owns cleanup
/// policy, and ending local lifetimes provides no secure erasure guarantee.
pub(super) fn compress(
    state: &mut [u64; 8],
    block: &[u8; 128],
    schedule_workspace: &mut [u64; 16],
) {
    let mut schedule = MessageSchedule::new(block, schedule_workspace);
    let mut working = *state;
    for &constant in &SHA_512_ROUND_CONSTANTS {
        let word = schedule.next_word().expect("one word per SHA-512 round");
        working = round(working, constant, word);
    }
    for (chaining, word) in state.iter_mut().zip(working) {
        *chaining = chaining.wrapping_add(word);
    }
}

/// Applies §6.4.2 step 3 to one `(a,b,c,d,e,f,g,h)` array in that order.
/// Every combination of working words, constant and schedule word is valid.
pub(super) fn round(
    [a, b, c, d, e, f, g, h]: [u64; 8],
    constant: u64,
    word: u64,
) -> [u64; 8] {
    let t1 = h
        .wrapping_add(big_sigma_1(e))
        .wrapping_add(ch(e, f, g))
        .wrapping_add(constant)
        .wrapping_add(word);
    let t2 = big_sigma_0(a).wrapping_add(maj(a, b, c));
    [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g]
}

#[cfg(test)]
#[path = "../../tests/sha512_tests/unit_tests/compression_unit_tests.rs"]
mod tests;
