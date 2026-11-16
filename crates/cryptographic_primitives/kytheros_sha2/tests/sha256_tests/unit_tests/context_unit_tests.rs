//! Functional evidence for incremental hashing and FIPS 180-4 padding.
//!
//! Published byte-oriented vectors check complete hashes. Local fixtures
//! from reference_vectors.py check arbitrary bits using a separate full
//! implementation with derived constants. Split equivalence alone is not
//! independent correctness evidence. Synthetic near-limit states exercise
//! arithmetic and encoding without claiming a 2^64-bit message was hashed.

use super::*;

fn expected_digest(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64);
    core::array::from_fn(|i| {
        u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()
    })
}

fn append(
    context: &mut Sha256Context,
    input: &[u8],
    offset_in_bits: usize,
    len_in_bits: usize,
    workspace: &mut [u32; 16],
) {
    let span = BitSpan::new(input, offset_in_bits, len_in_bits).unwrap();
    let old_len_in_bits = context.message_len_in_bits;
    context.update(span, workspace).unwrap();
    assert_eq!(
        context.message_len_in_bits,
        old_len_in_bits + len_in_bits as u64,
    );
    assert!(context.buffered_in_bits < 512);
    assert_eq!(
        context.buffered_in_bits as u64,
        context.message_len_in_bits % 512,
    );
}

// NIST CAVP SHA Test Vectors for Hashing Bit-Oriented Messages,
// shabittestvectors.zip, SHA256ShortMsg.rsp, Len = 0 (Msg = 00 is a
// placeholder, not an eight-bit message):
// https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/
// secure-hashing
// NIST worked examples, One Block Message Sample (abc) and Two Block
// Message Sample (the 448-bit string below):
// https://csrc.nist.gov/CSRC/media/Projects/
// Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf
#[test]
fn published_short_messages() {
    for (message, expected) in [
        (
            &b""[..],
            concat!(
                "e3b0c44298fc1c149afbf4c8996fb92427",
                "ae41e4649b934ca495991b7852b855",
            ),
        ),
        (
            &b"abc"[..],
            concat!(
                "ba7816bf8f01cfea414140de5dae2223",
                "b00361a396177a9cb410ff61f20015ad",
            ),
        ),
        (
            &b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"[..],
            concat!(
                "248d6a61d20638b8e5c026930c3e6039",
                "a33ce45964ff2167f6ecedd419db06c1",
            ),
        ),
    ] {
        let mut context = Sha256Context::new();
        let mut workspace = [0xdead_beef; 16];
        append(&mut context, message, 0, message.len() * 8, &mut workspace);
        assert_eq!(context.finalize(&mut workspace), expected_digest(expected));
    }
}

// NIST NSRL Test Data, SHA-256 digest of one million ASCII 'a' bytes:
// https://nist.gov/itl/ai/ai-standards-and-guidelines-group/nsrl-test-data
// Stream a fixed chunk 1000 times, without allocating the full message.
#[test]
fn published_million_a_multiblock_message() {
    let mut context = Sha256Context::new();
    let mut workspace = [u32::MAX; 16];
    for _ in 0..1000 {
        append(&mut context, &[b'a'; 1000], 0, 8000, &mut workspace);
    }
    assert_eq!(
        context.finalize(&mut workspace),
        expected_digest(concat!(
            "cdc76e5c9914fb9281a1c7e284d73e67",
            "f1809a48a497200e046d39ccc7112cd0",
        ))
    );
}

// Local, independently calculated fixtures, not NIST-published vectors.
// Reproduce: python3 context/reference_vectors.py (relative to sha256/).
// Input byte i = (37*i + 0x81) mod 256; source starts at bit offset 3.
const BIT_FIXTURES: &[(usize, &str)] = &[
    (
        0,
        concat!(
            "e3b0c44298fc1c149afbf4c8996fb924",
            "27ae41e4649b934ca495991b7852b855",
        ),
    ),
    (
        1,
        concat!(
            "bd4f9e98beb68c6ead3243b1b4c7fed7",
            "5fa4feaab1f84795cbd8a98676a2a375",
        ),
    ),
    (
        7,
        concat!(
            "415754792a5810a333ead32b0cfbc440",
            "442bd3126b4541f77166af4ff22ceb68",
        ),
    ),
    (
        8,
        concat!(
            "9d1e0e2d9459d06523ad13e28a4093c2",
            "316baafe7aec5b25f30eba2e113599c4",
        ),
    ),
    (
        446,
        concat!(
            "ee4258c284ce6dcc3925a28d28c6638a",
            "0231af3e606eedac7747868a4171727e",
        ),
    ),
    (
        447,
        concat!(
            "6d7d3e5c0e8c734b6ebac22154320ca3",
            "ee9b3955ae5be2f226d6ffd1c1b9fc11",
        ),
    ),
    (
        448,
        concat!(
            "b1c7196370959da3ffc51d728b968183",
            "278b43b9001ef535fa83705918bbfdc5",
        ),
    ),
    (
        449,
        concat!(
            "3f642b79635527dfff82aabe8fd690d4",
            "6d82427e9535165bf6227c7ed0c947e5",
        ),
    ),
    (
        510,
        concat!(
            "b0ab4dcd4af3ccc49f75168245ff3fb2",
            "c0a720422f3786bf2302f5e3d0d1f3b9",
        ),
    ),
    (
        511,
        concat!(
            "9eb6aee12e7de991d7caa697200f01e1",
            "9cc105010a6f8a0574dca9b3c30eaeab",
        ),
    ),
    (
        512,
        concat!(
            "4605cbc53cf112dad6c12cf2f8f44c26",
            "07e3efab0ede71c7258baad886f41176",
        ),
    ),
    (
        513,
        concat!(
            "952b21a146d31df7539a62a620fdfbf2",
            "b065366674a1694072efde49f7b52ec8",
        ),
    ),
    (
        1023,
        concat!(
            "10a59bf5188739ee512fb0e5f6358c4c",
            "cab1aea22b474706495f7cf432da4586",
        ),
    ),
    (
        1024,
        concat!(
            "2c4ed8b37c644541bd5e13352cee9a94",
            "3202748caca9562914b923d2ba5a8a5f",
        ),
    ),
];

fn source() -> [u8; 130] {
    core::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(0x81))
}

#[test]
fn arbitrary_bits_and_every_two_way_split_match_reference() {
    let input = source();
    for &(len_in_bits, expected) in BIT_FIXTURES {
        let expected = expected_digest(expected);
        for split_in_bits in 0..=len_in_bits {
            let mut context = Sha256Context::new();
            let mut workspace = [0x1234_5678; 16];
            append(&mut context, &input, 3, split_in_bits, &mut workspace);
            append(
                &mut context,
                &input,
                3 + split_in_bits,
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

#[test]
fn many_updates_relocate_source_bits_and_reuse_workspace() {
    let input = source();
    let mut workspace = [u32::MAX; 16];
    for &(len_in_bits, expected) in BIT_FIXTURES {
        for chunk_in_bits in [1, 7, 8, 31, 63, 447, 511, 512, 513] {
            let mut context = Sha256Context::new();
            // Alternate data sentinels to expose reliance on unused zeros.
            context.block.fill(0xa5);
            let mut consumed_in_bits = 0;
            while consumed_in_bits < len_in_bits {
                let count_in_bits =
                    chunk_in_bits.min(len_in_bits - consumed_in_bits);
                // Re-encode each chunk at all eight possible alignments
                // over time. The bits outside the span remain set.
                let offset_in_bits = consumed_in_bits % 8;
                let mut relocated = [0xff; 66];
                for i in 0..count_in_bits {
                    let source_in_bits = 3 + consumed_in_bits + i;
                    let bit = (input[source_in_bits / 8]
                        >> (7 - source_in_bits % 8))
                        & 1;
                    let dest_in_bits = offset_in_bits + i;
                    let mask = 0x80 >> (dest_in_bits % 8);
                    let byte = &mut relocated[dest_in_bits / 8];
                    *byte = (*byte & !mask) | (bit * mask);
                }
                append(
                    &mut context,
                    &relocated,
                    offset_in_bits,
                    count_in_bits,
                    &mut workspace,
                );
                consumed_in_bits += count_in_bits;
                let before = snapshot(&context);
                let workspace_before = workspace;
                append(&mut context, &[], 0, 0, &mut workspace);
                assert_eq!(snapshot(&context), before);
                assert_eq!(workspace, workspace_before);
            }
            // Arbitrary previous schedule values must never affect output.
            workspace.fill(0xabcdef01);
            assert_eq!(
                context.finalize(&mut workspace),
                expected_digest(expected)
            );
        }
    }
}

// This text oracle deliberately shares compression, but not padding or
// parsing logic. It isolates padding/length encoding from round correctness.
fn reference_finalization(context: &Sha256Context) -> [u8; 32] {
    let mut bits = std::string::String::new();
    for byte in context.block {
        bits.push_str(&std::format!("{byte:08b}"));
    }
    bits.truncate(context.buffered_in_bits);
    bits.push('1');
    let zero_count = (448 + 512 - bits.len()) % 512;
    bits.push_str(&"0".repeat(zero_count));
    bits.push_str(&std::format!("{:064b}", context.message_len_in_bits));
    let mut state = context.state;
    for text in bits.as_bytes().as_chunks::<512>().0 {
        let block = core::array::from_fn(|i| {
            let byte = core::str::from_utf8(&text[8 * i..8 * i + 8]).unwrap();
            u8::from_str_radix(byte, 2).unwrap()
        });
        compress(&mut state, &block, &mut [0; 16]);
    }
    let hex: std::string::String = state
        .iter()
        .map(|word| std::format!("{word:08x}"))
        .collect();
    expected_digest(&hex)
}

#[test]
fn padding_preserves_prefix_and_overwrites_every_suffix_bit() {
    // All remainders, including 446/447/448/449 and 510/511. Check
    // exact bits, not only digests; sentinel storage is deliberately dirty.
    for buffered_in_bits in 0..512 {
        for sentinel in [0, 0xff, 0xa5, 0x5a] {
            let mut block = [sentinel; 64];
            assert_eq!(
                pad_block(&mut block, buffered_in_bits),
                buffered_in_bits >= 448
            );
            for i in 0..512 {
                let expected = if i < buffered_in_bits {
                    (sentinel >> (7 - i % 8)) & 1
                } else {
                    u8::from(i == buffered_in_bits)
                };
                assert_eq!((block[i / 8] >> (7 - i % 8)) & 1, expected);
            }
        }
    }
}

#[test]
fn finalization_matches_text_padding_with_high_length_bytes() {
    for buffered_in_bits in 0..512 {
        // Synthetic compression state: no enormous message is processed.
        let mut context = Sha256Context::new();
        context.block = core::array::from_fn(|i| i as u8 ^ 0xa5);
        context.message_len_in_bits =
            0x0123_4567_89ab_c000 + buffered_in_bits as u64;
        context.buffered_in_bits = buffered_in_bits;
        let expected = reference_finalization(&context);
        assert_eq!(context.finalize(&mut [u32::MAX; 16]), expected);
    }
}

fn snapshot(context: &Sha256Context) -> ([u32; 8], [u8; 64], usize, u64) {
    (
        context.state,
        context.block,
        context.buffered_in_bits,
        context.message_len_in_bits,
    )
}

#[test]
fn length_limit_rejection_is_atomic_and_maximum_is_accepted() {
    let mut context = Sha256Context::new();
    // Synthetic but representation-consistent state, not a claim that
    // its chaining value was calculated from an actual near-limit message.
    context.message_len_in_bits = u64::MAX - 8;
    context.buffered_in_bits = 503;
    context.block.fill(0xa5);
    let mut workspace = [0xdead_beef; 16];
    let before = snapshot(&context);
    // A non-preflighted update would overwrite/compress before overflowing.
    let input = [0x3c; 128];
    assert_eq!(
        context.update(BitSpan::new(&input, 0, 1024).unwrap(), &mut workspace),
        Err(Sha256Error::MessageLengthOverflow)
    );
    assert_eq!(snapshot(&context), before);
    assert_eq!(workspace, [0xdead_beef; 16]);

    append(&mut context, &input, 0, 8, &mut workspace);
    assert_eq!(context.message_len_in_bits, u64::MAX);
    assert_eq!(context.buffered_in_bits, 511);
    let before = snapshot(&context);
    assert_eq!(
        context.update(BitSpan::new(&input, 0, 1).unwrap(), &mut workspace),
        Err(Sha256Error::MessageLengthOverflow)
    );
    assert_eq!(snapshot(&context), before);
    append(&mut context, &[], 0, 0, &mut workspace);
    assert_eq!(snapshot(&context), before);
    assert_eq!(workspace, [0xdead_beef; 16]);
    let expected = reference_finalization(&context);
    assert_eq!(context.finalize(&mut workspace), expected);
}

#[test]
fn initialization_and_immediate_block_compression() {
    let input = source();
    let mut context = Sha256Context::default();
    assert_eq!(context.message_len_in_bits, 0);
    assert_eq!(context.buffered_in_bits, 0);
    // FIPS 180-4 §5.3.3, independently written expectation.
    let mut expected = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];
    assert_eq!(context.state, expected);
    let block = input[..64].try_into().unwrap();
    compress(&mut expected, block, &mut [0; 16]);
    append(&mut context, &input, 0, 512, &mut [0; 16]);
    assert_eq!(context.state, expected);
    assert_eq!(context.buffered_in_bits, 0);
    append(&mut context, &input, 512, 1, &mut [u32::MAX; 16]);
    assert_eq!(context.state, expected);
    assert_eq!(context.buffered_in_bits, 1);
}
