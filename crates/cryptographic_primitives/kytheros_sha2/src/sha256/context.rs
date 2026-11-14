//! Incremental SHA-256 coordination, FIPS 180-4 (August 2015).
//!
//! Initialization follows §5.3.3, block processing §5.2.1 and §6.2.2,
//! and padding §5.1.1. The context retains only a partial block and chaining
//! values; it borrows input and schedule storage only during each call.

use super::{BitSpan, compress, pack_bits};

/// Failure to append a complete update to a SHA-256 message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sha256Error {
    /// The resulting logical message length would be at least 2^64 bits.
    MessageLengthOverflow,
}

/// Incremental SHA-256 working state with exactly one 64-byte block buffer.
///
/// At public update boundaries, `buffered_in_bits < 512` and
/// `buffered_in_bits == message_len_in_bits % 512`. The chaining words
/// represent all complete blocks already processed. Only the first
/// `buffered_in_bits` buffer bits are message data; all later bits may be
/// stale. Input and digest conventions follow FIPS 180-4 §§3.1–3.2:
/// MSb-first bits, big-endian block words and big-endian digest bytes.
///
/// This is a primitive working object, not an authoritative MCL context.
/// It owns no input references or schedule storage. Each operation borrows
/// sixteen caller-owned schedule words (64 bytes); their initial contents
/// are irrelevant and they can be reused across calls and contexts.
/// Compression also uses eight working words and scalar temporaries.
/// Source-level storage sizes are not a verified compiler stack bound.
/// All production paths use safe Rust without `std` or heap allocation.
///
/// Consuming finalization prevents reuse of this object. It does not
/// establish secure erasure of buffers, registers, stack or other copies.
/// Functional tests do not establish timing resistance, independent audit,
/// algorithm/module validation or production readiness.
///
/// # Example
///
/// ```
/// use kytheros_sha2::sha256::{BitSpan, Sha256Context};
///
/// let mut workspace = [0; 16];
/// let mut context = Sha256Context::new();
/// let input = BitSpan::new(b"abc", 0, 24).unwrap();
/// context.update(input, &mut workspace).unwrap();
/// let digest = context.finalize(&mut workspace);
/// assert_eq!(&digest[..4], &[0xba, 0x78, 0x16, 0xbf]);
/// ```
///
/// The fallible operations return distinct validation and length errors;
/// ordinary callers can handle each result with their own error policy.
pub struct Sha256Context {
    state: [u32; 8],
    block: [u8; 64],
    buffered_in_bits: usize,
    message_len_in_bits: u64,
}

impl Default for Sha256Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256Context {
    /// Initializes the eight chaining words from FIPS 180-4 §5.3.3.
    ///
    /// The message is empty, with no buffered bits. No external storage is
    /// accessed, and construction cannot fail or panic. SHA-224 has a
    /// different initial value and is not implemented by this context.
    pub const fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f,
                0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            block: [0; 64],
            buffered_in_bits: 0,
            message_len_in_bits: 0,
        }
    }

    /// Appends every logical bit of `input`, in order, to the message.
    ///
    /// [`BitSpan::new`] validates input bounds before this call. Updates
    /// may start and end at any bit position; only the concatenated logical
    /// bits determine the digest. Each full 512-bit block is immediately
    /// compressed, leaving fewer than 512 bits buffered. An empty update
    /// leaves both the context and workspace unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`Sha256Error::MessageLengthOverflow`] if the complete
    /// incoming length cannot be represented in `u64`, or if the new total
    /// would exceed `u64::MAX` (FIPS 180-4 §6.2). Validation precedes all
    /// writes, so rejection preserves every context field and the workspace.
    /// Valid spans within that limit cannot cause an internal packing error
    /// or panic: each pack starts below 512 and skips only consumed bits.
    /// The caller owns input validity, workspace reuse and cleanup.
    pub fn update(
        &mut self,
        mut input: BitSpan<'_>,
        schedule_workspace: &mut [u32; 16],
    ) -> Result<(), Sha256Error> {
        let incoming_len_in_bits = u64::try_from(input.len_in_bits())
            .map_err(|_| Sha256Error::MessageLengthOverflow)?;
        let new_len_in_bits = self
            .message_len_in_bits
            .checked_add(incoming_len_in_bits)
            .ok_or(Sha256Error::MessageLengthOverflow)?;

        while input.len_in_bits() != 0 {
            let progress =
                pack_bits(input, &mut self.block, self.buffered_in_bits)
                    .expect("validated span and destination below 512 bits");
            input = input.skip(progress.consumed_in_bits);
            self.buffered_in_bits = progress.destination_position_in_bits;
            if self.buffered_in_bits == 512 {
                compress(&mut self.state, &self.block, schedule_workspace);
                // Stale bits are overwritten by future input or padding.
                self.buffered_in_bits = 0;
            }
        }
        self.message_len_in_bits = new_len_in_bits;
        Ok(())
    }

    /// Consumes the context, pads the message and returns its SHA-256 digest.
    ///
    /// FIPS 180-4 §5.1.1 appends one set bit, the smallest number of zeros
    /// reaching position 448 modulo 512, and the original 64-bit length in
    /// big-endian order. Remainders below 448 need one final compression;
    /// remainders 448..=511 need two. Every padding bit is overwritten,
    /// including unused bits in a partial byte. The original message length
    /// is never increased to include padding. The same block buffer and
    /// caller-provided schedule workspace serve both final blocks.
    ///
    /// Per §6.2.2, the result is H_0 through H_7, each serialized with its
    /// most significant byte first, independently of host endianness.
    /// All valid contexts can be finalized without error or panic.
    /// No finalized-state flag or secure-erasure guarantee is provided.
    ///
    /// Ownership prevents reuse, including repeated finalization:
    ///
    /// ```compile_fail,E0382
    /// use kytheros_sha2::sha256::Sha256Context;
    /// let context = Sha256Context::new();
    /// let mut workspace = [0; 16];
    /// let _digest = context.finalize(&mut workspace);
    /// let _again = context.finalize(&mut workspace);
    /// ```
    pub fn finalize(mut self, schedule_workspace: &mut [u32; 16]) -> [u8; 32] {
        if pad_block(&mut self.block, self.buffered_in_bits) {
            compress(&mut self.state, &self.block, schedule_workspace);
            self.block.fill(0);
        }
        self.block[56..]
            .copy_from_slice(&self.message_len_in_bits.to_be_bytes());
        compress(&mut self.state, &self.block, schedule_workspace);
        let mut digest = [0; 32];
        for (bytes, word) in
            digest.as_chunks_mut::<4>().0.iter_mut().zip(self.state)
        {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }
}

/// Appends the set padding bit and clears the rest of the partial block.
/// Returns whether the length needs a second block. Requires fewer than
/// 512 buffered bits. Preserves every message bit, even in a partial byte.
/// The caller writes the length after any required extra compression.
fn pad_block(block: &mut [u8; 64], buffered_in_bits: usize) -> bool {
    let offset_in_bytes = buffered_in_bits / 8;
    let used_in_bits = buffered_in_bits % 8;
    // Preserve the high message bits, set the next bit, clear all below it.
    block[offset_in_bytes] &= !(0xff >> used_in_bits);
    block[offset_in_bytes] |= 0x80 >> used_in_bits;
    block[offset_in_bytes + 1..].fill(0);
    buffered_in_bits >= 448
}

#[cfg(test)]
mod tests;
