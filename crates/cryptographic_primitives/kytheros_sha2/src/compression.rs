//! Single-block SHA-256 compression, FIPS 180-4 §6.2.2 steps 1–4.

use super::schedule::MessageSchedule;
use super::{big_sigma_0, big_sigma_1, ch, maj};

// FIPS 180-4 §4.2.2, in round order.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

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
pub(crate) fn compress(
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

// FIPS 180-4 §6.2.2 step 3, using §4.1.2 equations (4.2)–(4.5).
fn round(
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
mod tests;
