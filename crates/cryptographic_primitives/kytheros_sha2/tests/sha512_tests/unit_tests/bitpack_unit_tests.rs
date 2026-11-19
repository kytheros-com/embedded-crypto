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
fn assert_block_text(block: &[u8; 128], expected: &str) {
    assert_eq!(expected.len(), 1024);
    for (byte_index, byte_text) in expected.as_bytes().chunks(8).enumerate() {
        let byte_text = core::str::from_utf8(byte_text).unwrap();
        let expected_byte = u8::from_str_radix(byte_text, 2).unwrap();
        assert_eq!(block[byte_index], expected_byte, "byte {byte_index}");
    }
}

// Covers all 8 x 8 alignments, lengths 1..=16, and both clear/set
// sentinel bits. These fixtures check the bit-packing contract.
#[test]
fn alignments_lengths_and_preservation() {
    for (source_offset_in_bits, input) in SOURCES.iter().enumerate() {
        for destination_offset_in_bits in 0..8 {
            for len_in_bits in 1..=16 {
                for (sentinel, sentinel_text) in
                    [(0xa5, "10100101"), (0x5a, "01011010")]
                {
                    let mut block = [sentinel; 128];
                    let source =
                        BitSpan::new(input, source_offset_in_bits, len_in_bits)
                            .unwrap();
                    let progress = pack_bits(
                        source,
                        &mut block,
                        destination_offset_in_bits,
                    )
                    .unwrap();
                    assert_eq!(progress.consumed_in_bits, len_in_bits);
                    let end_in_bits = destination_offset_in_bits + len_in_bits;
                    assert_eq!(
                        progress.destination_position_in_bits,
                        end_in_bits
                    );
                    let original = sentinel_text.repeat(128);
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
        for destination_offset_in_bits in 0..=1024 {
            let mut block = [0xa5; 128];
            let progress =
                pack_bits(source, &mut block, destination_offset_in_bits)
                    .unwrap();
            assert_eq!(progress.consumed_in_bits, 0);
            assert_eq!(
                progress.destination_position_in_bits,
                destination_offset_in_bits
            );
            assert_eq!(block, [0xa5; 128]);
        }
    }
}

#[test]
fn full_block_and_partial_copies() {
    // 1032 ones ensure even position zero must stop at the block end.
    let input = [0xff; 129];
    let source = BitSpan::new(&input, 0, 1032).unwrap();
    for destination_offset_in_bits in [0, 1, 895, 896, 1023, 1024] {
        let mut block = [0xa5; 128];
        let progress =
            pack_bits(source, &mut block, destination_offset_in_bits).unwrap();
        assert_eq!(
            progress.consumed_in_bits,
            1024 - destination_offset_in_bits
        );
        assert_eq!(progress.destination_position_in_bits, 1024);
        let original = "10100101".repeat(128);
        let expected = std::format!(
            "{}{}",
            &original[..destination_offset_in_bits],
            "1".repeat(1024 - destination_offset_in_bits)
        );
        assert_block_text(&block, &expected);
        // A second invocation at the returned full position cannot write.
        let full_block = block;
        assert_eq!(
            pack_bits(source, &mut block, 1024)
                .unwrap()
                .consumed_in_bits,
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
        for destination_offset_in_bits in [0, 1023, 1024] {
            let mut block = [0x5a; 128];
            let result = BitSpan::new(&input, offset_in_bits, len_in_bits)
                .and_then(|source| {
                    pack_bits(source, &mut block, destination_offset_in_bits)
                });
            assert_eq!(result, Err(error));
            assert_eq!(block, [0x5a; 128]);
        }
    }
    assert_eq!(BitSpan::new(&[], 0, 1), Err(SourceOutOfRange));
    assert_eq!(BitSpan::new(&[], 1, 0), Err(SourceOutOfRange));
    for len_in_bits in [0, 1] {
        let source = BitSpan::new(&input, 0, len_in_bits).unwrap();
        for destination_offset_in_bits in [1025, usize::MAX] {
            let mut block = [0x5a; 128];
            assert_eq!(
                pack_bits(source, &mut block, destination_offset_in_bits),
                Err(DestinationOutOfRange)
            );
            assert_eq!(block, [0x5a; 128]);
        }
    }
    // Capacity multiplication overflow is guarded in production but
    // cannot be exercised with a real slice of that size here in safe
    // Rust. Endpoint overflow above is exercised without fake slices.
}

#[test]
fn reuse_preserves_unwritten_bits_from_previous_block() {
    let mut block = [0x96; 128];
    let first = BitSpan::new(&[0x5a; 128], 0, 1024).unwrap();
    assert_eq!(
        pack_bits(first, &mut block, 0).unwrap().consumed_in_bits,
        1024
    );
    assert_eq!(block, [0x5a; 128]);

    let second = BitSpan::new(&[0xe3], 1, 6).unwrap();
    let progress = pack_bits(second, &mut block, 3).unwrap();
    assert_eq!(progress.consumed_in_bits, 6);
    assert_eq!(progress.destination_position_in_bits, 9);
    let expected = std::format!("010110001{}", &"01011010".repeat(128)[9..]);
    assert_block_text(&block, &expected);
}
