//! Incremental SHA-512 coordination, FIPS 180-4 (August 2015).
//!
//! Initialization follows §5.3.5, block processing §5.2.2 and §6.4.2,
//! and padding §5.1.2. The context retains only a partial block and chaining
//! values; it borrows input and schedule storage only during each call.

use super::BitSpan;
use super::bitpack::pack_bits;
use super::compression::compress;

/// Failure to append a complete update to a SHA-512 message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sha512Error {
    /// The resulting logical message length would be at least 2^128 bits.
    MessageLengthOverflow,
}

/// Incremental SHA-512 working state with exactly one 128-byte block buffer.
///
/// At public update boundaries, `buffered_in_bits < 1024` and
/// `buffered_in_bits == message_len_in_bits % 1024`. The chaining words
/// represent all complete blocks already processed. Only the first
/// `buffered_in_bits` buffer bits are message data; all later bits may be
/// stale. Input and digest conventions follow FIPS 180-4 §§3.1–3.2:
/// MSb-first bits, big-endian block words and big-endian digest bytes.
///
/// The MCL owns authoritative state and controls commits of this
/// primitive working copy.
/// It owns no input references or schedule storage. Each operation borrows
/// sixteen caller-owned schedule words (128 bytes); their initial contents
/// are irrelevant and they can be reused across calls and contexts.
/// Compression also uses eight working words and scalar temporaries.
/// Source-level storage sizes are not a verified compiler stack bound.
/// All production paths use safe Rust without `std` or heap allocation.
///
/// Consuming finalization prevents reuse of this object. It does not
/// establish secure erasure of buffers, registers, stack or other copies.
///
/// # Example
///
/// ```
/// use kytheros_sha2::sha512::{BitSpan, Sha512Context};
///
/// let mut workspace = [0; 16];
/// let mut context = Sha512Context::new();
/// let input = BitSpan::new(b"abc", 0, 24).unwrap();
/// context.update(input, &mut workspace).unwrap();
/// let digest = context.finalize(&mut workspace);
/// assert_eq!(&digest[..4], &[0xdd, 0xaf, 0x35, 0xa1]);
/// ```
///
/// The fallible operations return distinct validation and length errors;
/// ordinary callers can handle each result with their own error policy.
pub struct Sha512Context {
    state: [u64; 8],
    block: [u8; 128],
    buffered_in_bits: usize,
    message_len_in_bits: u128,
}

impl Default for Sha512Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha512Context {
    /// Initializes the eight chaining words from FIPS 180-4 §5.3.5.
    ///
    /// The message is empty, with no buffered bits. No external storage is
    /// accessed, and construction cannot fail or panic.
    pub const fn new() -> Self {
        Self {
            state: [
                0x6a09e667f3bcc908,
                0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b,
                0xa54ff53a5f1d36f1,
                0x510e527fade682d1,
                0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b,
                0x5be0cd19137e2179,
            ],
            block: [0; 128],
            buffered_in_bits: 0,
            message_len_in_bits: 0,
        }
    }

    /// Appends every logical bit of `input`, in order, to the message.
    ///
    /// [`BitSpan::new`] validates input bounds before this call. Updates
    /// may start and end at any bit position; only the concatenated logical
    /// bits determine the digest. Each full 1024-bit block is immediately
    /// compressed, leaving fewer than 1024 bits buffered. An empty update
    /// leaves both the context and workspace unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Sha512Error::MessageLengthOverflow`] if the new total
    /// would exceed `u128::MAX` (FIPS 180-4 §6.4). Validation precedes all
    /// writes, so rejection preserves every context field and the workspace.
    /// Valid spans within that limit cannot cause an internal packing error
    /// or panic: each pack starts below 1024 and skips only consumed bits.
    /// The caller owns input validity, workspace reuse and cleanup.
    pub fn update(
        &mut self,
        mut input: BitSpan<'_>,
        schedule_workspace: &mut [u64; 16],
    ) -> Result<(), Sha512Error> {
        let incoming_len_in_bits = input.len_in_bits() as u128;
        let new_len_in_bits = self
            .message_len_in_bits
            .checked_add(incoming_len_in_bits)
            .ok_or(Sha512Error::MessageLengthOverflow)?;

        while input.len_in_bits() != 0 {
            let progress =
                pack_bits(input, &mut self.block, self.buffered_in_bits)
                    .expect("validated span and destination below 1024 bits");
            input = input.skip(progress.consumed_in_bits);
            self.buffered_in_bits = progress.destination_position_in_bits;
            if self.buffered_in_bits == 1024 {
                compress(&mut self.state, &self.block, schedule_workspace);
                // Stale bits are overwritten by future input or padding.
                self.buffered_in_bits = 0;
            }
        }
        self.message_len_in_bits = new_len_in_bits;
        Ok(())
    }

    /// Consumes the context, pads the message and returns its SHA-512 digest.
    ///
    /// FIPS 180-4 §5.1.2 appends one set bit, the smallest number of zeros
    /// reaching position 896 modulo 1024, and the original 128-bit length in
    /// big-endian order. Remainders below 896 need one final compression;
    /// remainders 896..=1023 need two. Every padding bit is overwritten,
    /// including unused bits in a partial byte. The original message length
    /// is never increased to include padding. The same block buffer and
    /// caller-provided schedule workspace serve both final blocks.
    ///
    /// Per §6.4.2, the result is H_0 through H_7, each serialized with its
    /// most significant byte first, independently of host endianness.
    /// All valid contexts can be finalized without error or panic.
    ///
    /// Ownership prevents reuse, as shown by this compile-fail example.
    ///
    /// ```compile_fail,E0382
    /// use kytheros_sha2::sha512::Sha512Context;
    /// let context = Sha512Context::new();
    /// let mut workspace = [0; 16];
    /// let _digest = context.finalize(&mut workspace);
    /// let _again = context.finalize(&mut workspace);
    /// ```
    pub fn finalize(mut self, schedule_workspace: &mut [u64; 16]) -> [u8; 64] {
        let needs_second_block =
            pad_block(&mut self.block, self.buffered_in_bits);
        if needs_second_block {
            compress(&mut self.state, &self.block, schedule_workspace);
        }
        finish_padding(
            &mut self.block,
            needs_second_block,
            self.message_len_in_bits,
        );
        compress(&mut self.state, &self.block, schedule_workspace);
        let mut digest = [0; 64];
        for (bytes, word) in
            digest.as_chunks_mut::<8>().0.iter_mut().zip(self.state)
        {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }
}

/// Appends the set padding bit and clears the rest of the partial block.
/// Returns whether the length needs a second block. Requires fewer than
/// 1024 buffered bits. Preserves every message bit, even in a partial byte.
/// The caller writes the length after any required extra compression.
fn pad_block(block: &mut [u8; 128], buffered_in_bits: usize) -> bool {
    let offset_in_bytes = buffered_in_bits / 8;
    let used_in_bits = buffered_in_bits % 8;
    // Preserve the high message bits, set the next bit, clear all below it.
    block[offset_in_bytes] &= !(0xff >> used_in_bits);
    block[offset_in_bytes] |= 0x80 >> used_in_bits;
    block[offset_in_bytes + 1..].fill(0);
    buffered_in_bits >= 896
}

/// Writes the original bit length to the final padding block.
/// If the set padding bit occupied a previous block, clear this one first.
fn finish_padding(
    block: &mut [u8; 128],
    needs_second_block: bool,
    message_len_in_bits: u128,
) {
    if needs_second_block {
        block.fill(0);
    }
    block[112..].copy_from_slice(&message_len_in_bits.to_be_bytes());
}

#[cfg(test)]
#[path = "../../tests/sha512_tests/unit_tests/context_unit_tests.rs"]
mod tests;
