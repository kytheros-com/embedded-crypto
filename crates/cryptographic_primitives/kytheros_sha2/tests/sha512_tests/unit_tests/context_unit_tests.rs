//! Functional tests for SHA-512 bit input, streaming and finalization.
//!
//! Published complete-message digests and the independent Python fixtures
//! check the combined implementation. Text-based padding comparisons
//! check exact blocks, and synthetic lengths exercise encoding and
//! accounting without processing messages of those lengths.

use super::*;
use std::string::String;

fn expected_digest(hex: &str) -> [u8; 64] {
    assert_eq!(hex.len(), 128);
    core::array::from_fn(|i| {
        u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()
    })
}

fn append(
    context: &mut Sha512Context,
    input: &[u8],
    offset_in_bits: usize,
    len_in_bits: usize,
    workspace: &mut [u64; 16],
) {
    let span = BitSpan::new(input, offset_in_bits, len_in_bits).unwrap();
    let old_len_in_bits = context.message_len_in_bits;
    context.update(span, workspace).unwrap();
    assert_eq!(
        context.message_len_in_bits,
        old_len_in_bits + len_in_bits as u128
    );
    assert!(context.buffered_in_bits < 1024);
    assert_eq!(
        context.buffered_in_bits as u128,
        context.message_len_in_bits % 1024
    );
}

#[test]
fn published_complete_messages() {
    // Empty: tests/NIST_KAT_Files/shabytetestvectors.zip,
    // shabytetestvectors/SHA512ShortMsg.rsp, CAVS 11.0, Len = 0.
    // Msg = 00 is the placeholder for the empty message.
    // Other cases: NIST CSRC SHA512.pdf, One Block Message Sample (abc)
    // and Two Block Message Sample (112 bytes). See compression.rs for
    // the NIST examples index.
    let two_block_message = concat!(
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn",
        "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
    assert_eq!(two_block_message.len(), 112);
    for (message, expected) in [
        (
            &b""[..],
            concat!(
                "cf83e1357eefb8bdf1542850d66d8007d",
                "620e4050b5715dc83f4a921d36ce9ce4",
                "7d0d13c5d85f2b0ff8318d2877eec2f6",
                "3b931bd47417a81a538327af927da3e",
            ),
        ),
        (
            &b"abc"[..],
            concat!(
                "ddaf35a193617abacc417349ae204131",
                "12e6fa4e89a97ea20a9eeee64b55d39a",
                "2192992a274fc1a836ba3c23a3feebbd",
                "454d4423643ce80e2a9ac94fa54ca49f",
            ),
        ),
        (
            two_block_message.as_bytes(),
            concat!(
                "8e959b75dae313da8cf4f72814fc143f",
                "8f7779c6eb9f7fa17299aeadb6889018",
                "501d289e4900f7e4331b99dec4b5433a",
                "c7d329eeb6dd26545e96e55b874be909",
            ),
        ),
    ] {
        let mut context = Sha512Context::new();
        let mut workspace = [u64::MAX; 16];
        append(&mut context, message, 0, message.len() * 8, &mut workspace);
        let digest = context.finalize(&mut workspace);
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, expected_digest(expected));
    }
}

#[test]
fn published_million_a() {
    // NIST FIPS 180-2, Appendix C.3, SHA-512 Example (Long Message).
    // https://csrc.nist.gov/files/pubs/fips/180-2/final/docs/fips180-2.pdf
    let mut context = Sha512Context::new();
    let mut workspace = [u64::MAX; 16];
    let chunk = [b'a'; 1000];
    for _ in 0..1000 {
        append(&mut context, &chunk, 0, 8000, &mut workspace);
    }
    assert_eq!(
        context.finalize(&mut workspace),
        expected_digest(concat!(
            "e718483d0ce769644e2e42c7bc15b463",
            "8e1f98b13b2044285632a803afa973eb",
            "de0ff244877ea60a4cb0432ce577c31b",
            "eb009c5c2c49aa2e4eadb217ad8cc09b",
        ))
    );
}

// Local fixtures from the standalone context_reference_vectors.py.
// Logical input starts at bit 3 of this asymmetric backing array.
const SOURCE_OFFSET_IN_BITS: usize = 3;
const FIXTURES: &str = include_str!("context_bit_fixtures.txt");

fn source() -> [u8; 257] {
    core::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(0x81))
}

fn fixtures() -> impl Iterator<Item = (usize, [u8; 64])> {
    FIXTURES.lines().map(|row| {
        let (length, hex) = row.split_once(' ').unwrap();
        (length.parse().unwrap(), expected_digest(hex))
    })
}

#[test]
fn every_two_way_split_matches_independent_fixtures() {
    let input = source();
    for (len_in_bits, expected) in fixtures() {
        let mut whole = Sha512Context::new();
        let mut workspace = [u64::MAX; 16];
        append(
            &mut whole,
            &input,
            SOURCE_OFFSET_IN_BITS,
            len_in_bits,
            &mut workspace,
        );
        assert_eq!(whole.finalize(&mut workspace), expected);
        for split_in_bits in 0..=len_in_bits {
            let mut context = Sha512Context::new();
            append(
                &mut context,
                &input,
                SOURCE_OFFSET_IN_BITS,
                split_in_bits,
                &mut workspace,
            );
            append(
                &mut context,
                &input,
                SOURCE_OFFSET_IN_BITS + split_in_bits,
                len_in_bits - split_in_bits,
                &mut workspace,
            );
            assert_eq!(
                context.finalize(&mut workspace),
                expected,
                "length={len_in_bits}, split={split_in_bits}"
            );
        }
    }
}

fn bits_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| std::format!("{byte:08b}"))
        .collect()
}

fn text_byte(bits: &[u8]) -> u8 {
    u8::from_str_radix(core::str::from_utf8(bits).unwrap(), 2).unwrap()
}

#[test]
fn repeated_updates_relocate_bits_and_reuse_workspace() {
    let source_bits = bits_text(&source());
    let mut workspace = [u64::MAX; 16];
    for (len_in_bits, expected) in fixtures() {
        let start = SOURCE_OFFSET_IN_BITS;
        let message = &source_bits[start..start + len_in_bits];
        for chunk_in_bits in [1, 7, 8, 31, 63, 127, 895, 1023, 1024, 1025] {
            let mut context = Sha512Context::new();
            context.block.fill(0xa5);
            for (chunk_index, bits) in
                message.as_bytes().chunks(chunk_in_bits).enumerate()
            {
                // Text encoding rotates through all eight alignments and
                // sets bits outside each span to a deliberate sentinel.
                let offset_in_bits = (chunk_index + 1) % 8;
                let mut relocated = "1".repeat(offset_in_bits);
                relocated.push_str(core::str::from_utf8(bits).unwrap());
                let suffix_in_bits = (8 - relocated.len() % 8) % 8;
                relocated.push_str(&"1".repeat(suffix_in_bits));
                let bytes: std::vec::Vec<u8> =
                    relocated.as_bytes().chunks(8).map(text_byte).collect();
                append(
                    &mut context,
                    &bytes,
                    offset_in_bits,
                    bits.len(),
                    &mut workspace,
                );

                let before = snapshot(&context);
                let workspace_before = workspace;
                append(
                    &mut context,
                    &bytes,
                    bytes.len() * 8,
                    0,
                    &mut workspace,
                );
                assert_eq!(snapshot(&context), before);
                assert_eq!(workspace, workspace_before);
            }
            workspace.fill(0x1234_5678_9abc_def0);
            assert_eq!(
                context.finalize(&mut workspace),
                expected,
                "length={len_in_bits}, chunk={chunk_in_bits}"
            );
        }
    }
}

// A bitstring oracle assembles the message prefix and exact length field
// without the production byte masks, buffer clearing or integer encoding.
fn reference_padding(context: &Sha512Context) -> String {
    let mut bits = bits_text(&context.block);
    bits.truncate(context.buffered_in_bits);
    bits.push('1');
    let zeros_in_bits = (896 + 1024 - bits.len()) % 1024;
    bits.push_str(&"0".repeat(zeros_in_bits));
    bits.push_str(&std::format!("{:0128b}", context.message_len_in_bits));
    bits
}

fn assert_padding_blocks(context: &Sha512Context, expected: &str) {
    let mut block = context.block;
    let needs_second_block = pad_block(&mut block, context.buffered_in_bits);
    assert_eq!(needs_second_block, expected.len() == 2048);
    if needs_second_block {
        assert_eq!(bits_text(&block), &expected[..1024]);
    }
    finish_padding(&mut block, needs_second_block, context.message_len_in_bits);
    assert_eq!(bits_text(&block), &expected[expected.len() - 1024..]);
}

fn reference_finalization(context: &Sha512Context, padding: &str) -> [u8; 64] {
    // This oracle shares compression with the primitive to isolate padding
    // and digest serialization. The complete-message fixtures are separate.
    let mut state = context.state;
    for block in padding.as_bytes().as_chunks::<1024>().0 {
        let bytes =
            core::array::from_fn(|i| text_byte(&block[8 * i..8 * i + 8]));
        compress(&mut state, &bytes, &mut [0; 16]);
    }
    let hex: String = state
        .iter()
        .map(|word| std::format!("{word:016x}"))
        .collect();
    expected_digest(&hex)
}

#[test]
fn every_padding_remainder_preserves_prefix_and_encodes_length() {
    let asymmetric =
        core::array::from_fn(|i| (i as u8).wrapping_mul(37) ^ 0xa5);
    for buffered_in_bits in 0..1024 {
        for block in [[0; 128], [0xff; 128], asymmetric] {
            let mut context = Sha512Context::new();
            context.block = block;
            context.buffered_in_bits = buffered_in_bits;
            // Synthetic length with a populated high half and matching
            // low ten bits. Chaining state is a fixture.
            context.message_len_in_bits =
                (0x0123_4567_89ab_cdef_u128 << 64) + buffered_in_bits as u128;
            let padding = reference_padding(&context);
            assert_padding_blocks(&context, &padding);
            let expected = reference_finalization(&context, &padding);
            assert_eq!(context.finalize(&mut [u64::MAX; 16]), expected);
        }
    }
}

#[test]
fn high_half_length_encoding() {
    for message_len_in_bits in [
        u64::MAX as u128,
        1_u128 << 64,
        (1_u128 << 64) + 1,
        u128::MAX - 128,
        u128::MAX - 127,
        u128::MAX - 1,
        u128::MAX,
    ] {
        let mut context = Sha512Context::new();
        context.block.fill(0xa5);
        context.message_len_in_bits = message_len_in_bits;
        context.buffered_in_bits = (message_len_in_bits % 1024) as usize;
        let padding = reference_padding(&context);
        assert_padding_blocks(&context, &padding);
        let expected = reference_finalization(&context, &padding);
        assert_eq!(context.finalize(&mut [u64::MAX; 16]), expected);
    }
}

fn snapshot(context: &Sha512Context) -> ([u64; 8], [u8; 128], usize, u128) {
    (
        context.state,
        context.block,
        context.buffered_in_bits,
        context.message_len_in_bits,
    )
}

#[test]
fn length_overflow_is_atomic_and_maximum_is_accepted() {
    let mut context = Sha512Context::new();
    context.message_len_in_bits = u128::MAX - 8;
    context.buffered_in_bits = 1015;
    context.block.fill(0xa5);
    let mut workspace = [0xdead_beef; 16];
    let input = [0x3c; 128];

    let before = snapshot(&context);
    // Without preflight this would fill a block and mutate the workspace.
    let too_long = BitSpan::new(&input, 0, 1024).unwrap();
    assert_eq!(
        context.update(too_long, &mut workspace),
        Err(Sha512Error::MessageLengthOverflow)
    );
    assert_eq!(snapshot(&context), before);
    assert_eq!(workspace, [0xdead_beef; 16]);

    append(&mut context, &input, 0, 8, &mut workspace);
    assert_eq!(context.message_len_in_bits, u128::MAX);
    assert_eq!(context.buffered_in_bits, 1023);
    let before = snapshot(&context);
    let bit = BitSpan::new(&input, 0, 1).unwrap();
    assert_eq!(
        context.update(bit, &mut workspace),
        Err(Sha512Error::MessageLengthOverflow)
    );
    assert_eq!(snapshot(&context), before);
    assert_eq!(workspace, [0xdead_beef; 16]);

    append(&mut context, &[], 0, 0, &mut workspace);
    assert_eq!(snapshot(&context), before);
    assert_eq!(workspace, [0xdead_beef; 16]);
}

#[test]
fn update_carries_into_high_half() {
    let mut context = Sha512Context::new();
    context.message_len_in_bits = u64::MAX as u128;
    context.buffered_in_bits = 1023;
    let mut expected = context.state;
    let mut block = context.block;
    block[127] = 1;
    compress(&mut expected, &block, &mut [0; 16]);
    append(&mut context, &[0x80], 0, 1, &mut [u64::MAX; 16]);
    assert_eq!(context.message_len_in_bits, 1_u128 << 64);
    assert_eq!(context.buffered_in_bits, 0);
    assert_eq!(context.state, expected);
}

#[test]
fn initialization_and_immediate_compression() {
    // NIST FIPS 180-4 §5.3.5, H_0 through H_7.
    let iv = [
        0x6a09_e667_f3bc_c908,
        0xbb67_ae85_84ca_a73b,
        0x3c6e_f372_fe94_f82b,
        0xa54f_f53a_5f1d_36f1,
        0x510e_527f_ade6_82d1,
        0x9b05_688c_2b3e_6c1f,
        0x1f83_d9ab_fb41_bd6b,
        0x5be0_cd19_137e_2179,
    ];
    let mut context = Sha512Context::default();
    assert_eq!(snapshot(&context), (iv, [0; 128], 0, 0));
    assert_eq!(snapshot(&context), snapshot(&Sha512Context::new()));

    let first = [0x6d; 128];
    let second = [0xb3; 128];
    let mut workspace = [u64::MAX; 16];
    let mut expected = iv;
    append(&mut context, &first, 0, 1024, &mut workspace);
    compress(&mut expected, &first, &mut [0; 16]);
    assert_eq!(context.state, expected);
    assert_eq!(context.buffered_in_bits, 0);

    let workspace_before = workspace;
    append(&mut context, &second, 0, 13, &mut workspace);
    assert_eq!(context.state, expected);
    assert_eq!(context.buffered_in_bits, 13);
    assert_eq!(context.block[0], second[0]);
    assert_eq!(context.block[1] >> 3, second[1] >> 3);
    assert_eq!(workspace, workspace_before);

    append(&mut context, &second, 13, 1011, &mut workspace);
    compress(&mut expected, &second, &mut [0; 16]);
    assert_eq!(context.state, expected);
    assert_eq!(context.buffered_in_bits, 0);
}
