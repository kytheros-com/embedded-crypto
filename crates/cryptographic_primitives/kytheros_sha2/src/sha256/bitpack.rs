//! SHA-256 input bit packing as specified in NIST FIPS 180-4.
//!
//! # Scope and algorithm decomposition
//!
//! This draft implements input-span validation and single-block bit packing
//!
//! [`BitSpan`] and [`pack_bits`] traverse bytes in slice
//! order and bits MSb-first.
//!


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
}

/// Progress from one call to [`pack_bits`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackingProgress {
    /// Number of leading logical source bits written by this call.
    pub consumed_in_bits: usize,
    /// First unwritten destination position, in the inclusive range 0..=512.
    pub destination_position_in_bits: usize,
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
pub fn pack_bits(
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
mod tests {
    use super::*;

    // Local fixtures, not NIST vectors: the logical string is
    // 1011001101101101 (hex B36D), shifted right by each source offset.
    // Ones outside the span deliberately differ between representations.
    const SOURCES: [[u8; 3]; 8] = [
        [0xb3, 0x6d, 0xff],
        [0xd9, 0xb6, 0xff],
        [0xec, 0xdb, 0x7f],
        [0xf6, 0x6d, 0xbf],
        [0xfb, 0x36, 0xdf],
        [0xfd, 0x9b, 0x6f],
        [0xfe, 0xcd, 0xb7],
        [0xff, 0x66, 0xdb],
    ];
    const LOGICAL_BITS: &str = "1011001101101101";

    // An independent text oracle: concatenate textual bits and decode
    // complete groups with radix parsing, rather than source extraction
    // and destination masks. Comparing the entire block checks untouched
    // prefix/suffix bits, including partial bytes. Test-only allocation
    // belongs to the hosted harness, not to the portable implementation.
    fn assert_block_text(block: &[u8; 64], expected: &str) {
        assert_eq!(expected.len(), 512);
        for (byte_index, byte_text) in expected.as_bytes().chunks(8).enumerate()
        {
            let byte_text = core::str::from_utf8(byte_text).unwrap();
            let expected_byte = u8::from_str_radix(byte_text, 2).unwrap();
            assert_eq!(block[byte_index], expected_byte, "byte {byte_index}");
        }
    }

    // Covers all 8 x 8 alignments, lengths 1..=16, and both clear/set
    // sentinel bits. These finite fixtures are functional evidence only;
    // they do not prove correctness for all strings, timing, or full SHA-256.
    #[test]
    fn alignments_lengths_and_preservation() {
        for (source_offset_in_bits, input) in SOURCES.iter().enumerate() {
            for destination_offset_in_bits in 0..8 {
                for len_in_bits in 1..=16 {
                    for (sentinel, sentinel_text) in
                        [(0xa5, "10100101"), (0x5a, "01011010")]
                    {
                        let mut block = [sentinel; 64];
                        let source = BitSpan::new(
                            input,
                            source_offset_in_bits,
                            len_in_bits,
                        )
                        .unwrap();
                        let progress = pack_bits(
                            source,
                            &mut block,
                            destination_offset_in_bits,
                        )
                        .unwrap();
                        assert_eq!(progress.consumed_in_bits, len_in_bits);
                        let end_in_bits =
                            destination_offset_in_bits + len_in_bits;
                        assert_eq!(
                            progress.destination_position_in_bits,
                            end_in_bits
                        );
                        let original = sentinel_text.repeat(64);
                        let expected = std::format!(
                            "{}{}{}",
                            &original[..destination_offset_in_bits],
                            &LOGICAL_BITS[..len_in_bits],
                            &original[end_in_bits..]
                        );
                        assert_block_text(&block, &expected);
                    }
                }
            }
        }
    }

    #[test]
    fn empty_spans_preserve_storage() {
        for (input, offset_in_bits) in
            [(&[][..], 0), (&[0xff][..], 0), (&[0xff][..], 8)]
        {
            let source = BitSpan::new(input, offset_in_bits, 0).unwrap();
            for destination_offset_in_bits in 0..=512 {
                let mut block = [0xa5; 64];
                let progress =
                    pack_bits(source, &mut block, destination_offset_in_bits)
                        .unwrap();
                assert_eq!(progress.consumed_in_bits, 0);
                assert_eq!(
                    progress.destination_position_in_bits,
                    destination_offset_in_bits
                );
                assert_eq!(block, [0xa5; 64]);
            }
        }
    }

    #[test]
    fn full_block_and_partial_copies() {
        // 520 ones ensure even position zero must stop at the block end.
        let input = [0xff; 65];
        let source = BitSpan::new(&input, 0, 520).unwrap();
        for destination_offset_in_bits in [0, 447, 448, 511, 512] {
            let mut block = [0xa5; 64];
            let progress =
                pack_bits(source, &mut block, destination_offset_in_bits)
                    .unwrap();
            assert_eq!(
                progress.consumed_in_bits,
                512 - destination_offset_in_bits
            );
            assert_eq!(progress.destination_position_in_bits, 512);
            let original = "10100101".repeat(64);
            let expected = std::format!(
                "{}{}",
                &original[..destination_offset_in_bits],
                "1".repeat(512 - destination_offset_in_bits)
            );
            assert_block_text(&block, &expected);
            // A second invocation at the returned full position cannot write.
            let full_block = block;
            assert_eq!(
                pack_bits(source, &mut block, 512).unwrap().consumed_in_bits,
                0
            );
            assert_eq!(block, full_block);
        }
    }

    #[test]
    fn rejected_operations_preserve_storage() {
        use PackingError::*;
        let input = [0x80];
        for (offset_in_bits, len_in_bits, error) in [
            (0, 9, SourceOutOfRange),
            (8, 1, SourceOutOfRange),
            (9, 0, SourceOutOfRange),
            (usize::MAX, 0, SourceOutOfRange),
            (usize::MAX, 1, ArithmeticOverflow),
            (1, usize::MAX, ArithmeticOverflow),
        ] {
            for destination_offset_in_bits in [0, 511, 512] {
                let mut block = [0x5a; 64];
                let result = BitSpan::new(&input, offset_in_bits, len_in_bits)
                    .and_then(|source| {
                        pack_bits(
                            source,
                            &mut block,
                            destination_offset_in_bits,
                        )
                    });
                assert_eq!(result, Err(error));
                assert_eq!(block, [0x5a; 64]);
            }
        }
        assert_eq!(BitSpan::new(&[], 0, 1), Err(SourceOutOfRange));
        assert_eq!(BitSpan::new(&[], 1, 0), Err(SourceOutOfRange));
        for len_in_bits in [0, 1] {
            let source = BitSpan::new(&input, 0, len_in_bits).unwrap();
            for destination_offset_in_bits in [513, usize::MAX] {
                let mut block = [0x5a; 64];
                assert_eq!(
                    pack_bits(source, &mut block, destination_offset_in_bits),
                    Err(DestinationOutOfRange)
                );
                assert_eq!(block, [0x5a; 64]);
            }
        }
        // Capacity multiplication overflow is guarded in production but
        // cannot be exercised with a real slice of that size here in safe
        // Rust. Endpoint overflow above is exercised without fake slices.
    }
}
