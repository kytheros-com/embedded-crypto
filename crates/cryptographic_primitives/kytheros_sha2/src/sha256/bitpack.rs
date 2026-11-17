//! SHA-256 input-span validation and single-block bit packing.
//!
//! Source offsets and logical lengths are measured in bits; source and
//! destination traverse bytes in slice order and bits MSb-first, following
//! FIPS 180-4 (August 2015) §3.1. Packing preserves partial-byte bits outside
//! the written interval. API documentation defines bounds, errors, progress
//! and caller-owned block reuse; padding is not implemented here.
//!
//! Tests cover all source/destination bit alignments for local text
//! fixtures, empty spans, block boundaries, partial copies and rejection
//! without writes. They do not establish complete hashing or timing
//! properties; slice-capacity multiplication overflow cannot be exercised
//! with a real slice of the required size in the safe hosted harness.

/// A validated borrowed bitstring, traversed MSb-first within each byte.
///
/// Logical bit `i` is bit `7 - ((offset_in_bits + i) % 8)` of byte
/// `(offset_in_bits + i) / 8`. Bytes occur in slice order, independent of
/// native endianness (FIPS 180-4 §3.1). No input bytes are copied.
/// Fields are private so construction preserves the validated bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitSpan<'a> {
    input: &'a [u8],
    offset_in_bits: usize,
    len_in_bits: usize,
}

/// Rejection reasons for bitstring construction and block packing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackingError {
    /// The slice capacity or source endpoint overflowed `usize`.
    ArithmeticOverflow,
    /// The source endpoint exceeds the slice's bit capacity.
    SourceOutOfRange,
    /// The destination position exceeds 512 bits.
    DestinationOutOfRange,
}

impl<'a> BitSpan<'a> {
    /// Validates the entire span before allowing any use by the packer.
    ///
    /// Both `input.len() * 8` and `offset_in_bits + len_in_bits` use
    /// checked arithmetic. Overflow returns
    /// [`PackingError::ArithmeticOverflow`];
    /// an endpoint beyond capacity returns [`PackingError::SourceOutOfRange`].
    /// A zero-length span may start anywhere through the capacity,
    /// inclusive; an empty slice permits only offset zero and length zero.
    /// This operation does not panic, allocate, or modify storage.
    pub fn new(
        input: &'a [u8],
        offset_in_bits: usize,
        len_in_bits: usize,
    ) -> Result<Self, PackingError> {
        let capacity_in_bits = input
            .len()
            .checked_mul(8)
            .ok_or(PackingError::ArithmeticOverflow)?;
        let end_in_bits = offset_in_bits
            .checked_add(len_in_bits)
            .ok_or(PackingError::ArithmeticOverflow)?;
        if end_in_bits > capacity_in_bits {
            return Err(PackingError::SourceOutOfRange);
        }
        Ok(Self {
            input,
            offset_in_bits,
            len_in_bits,
        })
    }

    /// Returns the validated logical length, not the backing slice capacity.
    pub(super) const fn len_in_bits(self) -> usize {
        self.len_in_bits
    }

    /// Removes a validated prefix; the remaining endpoint is unchanged.
    /// Panics if the requested prefix exceeds the logical length.
    pub(super) fn skip(self, consumed_in_bits: usize) -> Self {
        assert!(consumed_in_bits <= self.len_in_bits);
        Self {
            input: self.input,
            offset_in_bits: self.offset_in_bits + consumed_in_bits,
            len_in_bits: self.len_in_bits - consumed_in_bits,
        }
    }
}

/// Progress from one call to [`pack_bits`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PackingProgress {
    /// Number of leading logical source bits written by this call.
    pub(super) consumed_in_bits: usize,
    /// First unwritten destination position, in the inclusive range 0..=512.
    pub(super) destination_position_in_bits: usize,
}

/// Copies a validated bitstring into one caller-owned 512-bit block.
///
/// With initial destination position `d`, writes exactly
/// `n = min(source.len_in_bits, 512 - d)` bits to `[d, d + n)` and
/// preserves every other bit. Source and destination use MSb-first bit
/// order and slice byte order, following FIPS 180-4 §3.1. The block size
/// follows §5.2.1; this function does not parse blocks into words.
///
/// Positions 0..=512 are accepted. Position 512 returns zero consumption
/// without modifying the full block. The caller must process the block
/// before choosing a new position for reuse; this stateless API cannot
/// enforce that lifecycle. It never clears or automatically reuses a block.
/// Empty input also succeeds without modification at every valid position.
///
/// # Errors
///
/// A position above 512 returns [`PackingError::DestinationOutOfRange`]
/// before any write. Invalid source spans cannot be constructed through
/// [`BitSpan::new`]. All rejections leave destination storage unchanged.
///
/// # Invariants and scope
///
/// Before iteration `i`, exactly `[d, d + i)` has been copied. Validated
/// source bounds and `i < n` bound every index and addition; no input can
/// cause arithmetic overflow or an out-of-bounds panic. At most 512 loop
/// iterations run, using scalar temporaries and no additional block buffer.
/// No allocation, native-endian conversion, MCL access, or retained state
/// is involved. The caller owns all working storage and cleanup duties.
/// This is input packing only: no padding or other hashing stage is done.
pub(super) fn pack_bits(
    source: BitSpan<'_>,
    block: &mut [u8; 64],
    destination_offset_in_bits: usize,
) -> Result<PackingProgress, PackingError> {
    if destination_offset_in_bits > 512 {
        return Err(PackingError::DestinationOutOfRange);
    }
    let consumed_in_bits =
        source.len_in_bits.min(512 - destination_offset_in_bits);
    for i in 0..consumed_in_bits {
        let source_position_in_bits = source.offset_in_bits + i;
        let destination_position_in_bits = destination_offset_in_bits + i;
        let bit = (source.input[source_position_in_bits / 8]
            >> (7 - source_position_in_bits % 8))
            & 1;
        let mask = 1 << (7 - destination_position_in_bits % 8);
        let byte = &mut block[destination_position_in_bits / 8];
        *byte = (*byte & !mask) | (bit * mask);
    }
    Ok(PackingProgress {
        consumed_in_bits,
        destination_position_in_bits: destination_offset_in_bits
            + consumed_in_bits,
    })
}

#[cfg(test)]
#[path = "../../tests/sha256_tests/unit_tests/bitpack_unit_tests.rs"]
mod tests;
