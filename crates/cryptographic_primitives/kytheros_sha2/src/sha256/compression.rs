//! Single-block SHA-256 compression, FIPS 180-4 (August 2015), §6.2.2.
//!
//! Step 3 computes `T1 = h + Sigma_1(e) + Ch(e,f,g) + K_t + W_t` and
//! `T2 = Sigma_0(a) + Maj(a,b,c)`, then replaces `(a,b,c,d,e,f,g,h)` by
//! `(T1+T2,a,b,c,d+T1,e,f,g)`. All additions are modulo 2^32. Step 4
//! adds the final working words to the original chaining words. The API
//! below documents working-copy ownership, round invariants and workspace.
//!
//! Tests compare a direct 64-word schedule and alternative Boolean formulas
//! across structured, single-bit and deterministic sampled blocks and four
//! initial states. Round checkpoints and sequential-block workspace reuse
//! are checked. The reference shares the production constants, so it cannot
//! independently verify them. A manually padded ASCII "abc" block checks
//! the combined result against the published [NIST NSRL test data][nsrl].
//! This exercises compression, not an implemented padding or hashing API;
//! sampled agreement does not establish exhaustive correctness, embedded
//! execution, algorithm validation or module-level assurance.
//!
//! [nsrl]:
//! https://nist.gov/itl/ai/ai-standards-and-guidelines-group/nsrl-test-data

use super::{
    MessageSchedule, SHA_256_ROUND_CONSTANTS as K, big_sigma_0, big_sigma_1,
    ch, maj,
};

/// Applies FIPS 180-4 §6.2.2 to one complete 512-bit block.
///
/// The block is read in byte order, MSb-first within each byte, and decoded
/// into big-endian words. State words are numeric values in H_0..H_7 order.
/// The state is a caller-owned working copy. MCL owns authoritative state.
///
/// The schedule overwrites all 16 workspace words (64 bytes). Compression
/// keeps eight working words (32 bytes), a schedule cursor and scalar
/// temporaries, with no block copy or dynamic allocation. The caller owns
/// workspace reuse and cleanup.
///
/// Before round t, the working words contain the result of the previous t
/// rounds, while the input chaining words still hold the original values.
/// Feed-forward updates all eight chaining words only after round 63.
/// Every word addition wraps modulo 2^32 (§3.2). All input values are valid.
/// A fresh schedule supplies exactly one word for each of the 64 constants,
/// so the exhaustion check cannot fail with the current schedule contract.
pub(super) fn compress(
    state: &mut [u32; 8],
    block: &[u8; 64],
    schedule_workspace: &mut [u32; 16],
) {
    let mut schedule = MessageSchedule::new(block, schedule_workspace);
    let mut working = *state;
    for constant in K {
        let word = schedule.next_word().expect("one word per SHA-256 round");
        working = round(working, constant, word);
    }
    for (chaining, word) in state.iter_mut().zip(working) {
        *chaining = chaining.wrapping_add(word);
    }
}

/// One round, FIPS 180-4 §6.2.2 step 3 and §4.1.2 Eqs. (4.2)–(4.5).
/// Accepts all working words, constants and schedule words without panic.
pub(super) fn round(
    [a, b, c, d, e, f, g, h]: [u32; 8],
    constant: u32,
    word: u32,
) -> [u32; 8] {
    let t1 = h
        .wrapping_add(big_sigma_1(e))
        .wrapping_add(ch(e, f, g))
        .wrapping_add(constant)
        .wrapping_add(word);
    let t2 = big_sigma_0(a).wrapping_add(maj(a, b, c));
    [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g]
}

#[cfg(test)]
#[path = "../../tests/sha256_tests/unit_tests/compression_unit_tests.rs"]
mod tests;
