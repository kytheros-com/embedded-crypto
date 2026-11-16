//! Kytheros-specific partitioning stress, not a NIST-prescribed procedure.
//! Official seeds/checkpoints and SHAVS recurrence are shared unchanged.
//! Logical fragments traverse the original serialized message MSb first.

#[path = "support/monte.rs"]
mod monte;

use kytheros_sha2::sha256::{BitSpan, Sha256Context};
use std::time::Instant;

const IRREGULAR: [usize; 9] = [1, 3, 5, 7, 9, 13, 17, 31, 63];

// A finite schedule permits exhaustive independent partition verification.
// Slot = (COUNT * 1000 + i - 3) modulo 32, identical for both collections.
fn patterns() -> Vec<Vec<usize>> {
    let mut result = vec![vec![1; 768], vec![3; 256]];
    result.push(irregular(0));
    result.push(vec![13, 499, 1, 255]);
    for first_in_bits in [1, 767, 447, 448, 449, 511, 512, 513] {
        result.push(vec![first_in_bits, 768 - first_in_bits]);
    }
    for rotation in 0..20 {
        result.push(irregular(rotation % IRREGULAR.len()));
    }
    assert_eq!(result.len(), 32);
    result
}

fn irregular(rotation: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut remaining_in_bits = 768;
    let mut index = rotation;
    while remaining_in_bits > 0 {
        let len_in_bits =
            IRREGULAR[index % IRREGULAR.len()].min(remaining_in_bits);
        result.push(len_in_bits);
        remaining_in_bits -= len_in_bits;
        index += 1;
    }
    result
}

#[derive(Default, Debug)]
struct Counts {
    updates: usize,
    empty_updates: usize,
    boundaries: usize,
    unaligned_boundaries: usize,
    relocated_messages: usize,
    differential: usize,
    empty_equivalence: usize,
    supplemental_updates: usize,
    // Endpoints include the final 768-bit endpoint, separately from the
    // internal-boundary fraction (which excludes endpoints and empties).
    endpoints: [usize; 8],
}

const ENDPOINTS: [usize; 8] = [447, 448, 449, 511, 512, 513, 767, 768];

fn bit(input: &[u8], position_in_bits: usize) -> u8 {
    (input[position_in_bits / 8] >> (7 - position_in_bits % 8)) & 1
}

// Test-only relocation with a caller-owned buffer; no production pack_bits.
fn relocate(
    message: &[u8; 96],
    offset_in_bits: usize,
    len_in_bits: usize,
    source_offset_in_bits: usize,
    storage: &mut [u8; 97],
) {
    storage.fill(0xa5);
    for index in 0..len_in_bits {
        let destination_in_bits = source_offset_in_bits + index;
        let mask = 0x80 >> (destination_in_bits % 8);
        let byte = &mut storage[destination_in_bits / 8];
        *byte &= !mask;
        if bit(message, offset_in_bits + index) != 0 {
            *byte |= mask;
        }
    }
}

fn fragmented(
    message: &[u8; 96],
    lengths_in_bits: &[usize],
    empty: bool,
    relocated: bool,
    counts: &mut Counts,
) -> [u8; 32] {
    assert!(lengths_in_bits.len() >= 2);
    let mut context = Sha256Context::new();
    let mut workspace = [0u32; 16];
    let mut storage = [0u8; 97];
    let mut offset_in_bits = 0usize;
    let mut update = |source: &[u8], start_in_bits, len_in_bits| {
        let span = BitSpan::new(source, start_in_bits, len_in_bits).unwrap();
        context.update(span, &mut workspace).unwrap();
        counts.updates += 1;
        counts.empty_updates += usize::from(len_in_bits == 0);
    };
    if empty {
        update(message, 0, 0);
    }
    for (ordinal, &len_in_bits) in lengths_in_bits.iter().enumerate() {
        assert!(len_in_bits > 0);
        let end_in_bits = offset_in_bits.checked_add(len_in_bits).unwrap();
        assert!(end_in_bits <= 768);
        if relocated {
            let source_offset_in_bits = 1 + ordinal % 8;
            relocate(
                message,
                offset_in_bits,
                len_in_bits,
                source_offset_in_bits,
                &mut storage,
            );
            update(&storage, source_offset_in_bits, len_in_bits);
        } else {
            update(message, offset_in_bits, len_in_bits);
        }
        if empty && ordinal == 0 {
            update(message, end_in_bits, 0);
        }
        offset_in_bits = end_in_bits;
    }
    assert_eq!(offset_in_bits, 768);
    if empty {
        update(message, 768, 0);
    }
    // Count only internal boundaries between nonempty fragments.
    let mut endpoint_in_bits = 0;
    for &len_in_bits in lengths_in_bits {
        endpoint_in_bits += len_in_bits;
        for (index, endpoint) in ENDPOINTS.iter().enumerate() {
            counts.endpoints[index] +=
                usize::from(*endpoint == endpoint_in_bits);
        }
        if endpoint_in_bits < 768 {
            counts.boundaries += 1;
            counts.unaligned_boundaries +=
                usize::from(endpoint_in_bits % 8 != 0);
        }
    }
    counts.relocated_messages += usize::from(relocated);
    context.finalize(&mut workspace)
}

fn verify(collection: &str) {
    let started = Instant::now();
    let response_file = format!("{collection}/SHA256Monte.rsp");
    let detail_file = format!("{collection}/SHA256Monte.txt");
    let response = monte::load(&response_file, false);
    let details = monte::load(&detail_file, true);
    assert_eq!(response.seed, details.seed);
    let patterns = patterns();
    let mut seed = response.seed;
    let mut counts = Counts::default();
    let mut hashes = 0;
    let mut intermediate_pairs = 0;
    for (count, checkpoint) in response.checkpoints.iter().enumerate() {
        monte::compare(
            &detail_file,
            count,
            &checkpoint.digest,
            &details.checkpoints[count].digest,
        );
        let mut inner_index = 3;
        seed = monte::outer(
            seed,
            |message| {
                let i = inner_index;
                inner_index += 1;
                let serial = count * 1000 + i - 3;
                let pattern = serial % 32;
                let lengths = &patterns[pattern];
                let empty = serial % 64 == 0;
                let relocated = serial % 1000 == 0;
                // Formatting and allocation occur only if an assertion fails.
                let result = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| {
                        let digest = fragmented(
                            message,
                            lengths,
                            empty,
                            relocated,
                            &mut counts,
                        );
                        hashes += 1;
                        if i <= 7 {
                            let (expected_message, expected_digest) =
                                &details.checkpoints[count].intermediate[i - 3];
                            assert_eq!(message, expected_message);
                            assert_eq!(&digest, expected_digest);
                            intermediate_pairs += 1;
                        }
                        if i <= 7 || i == 1002 || empty {
                            assert_eq!(digest, monte::hash(message));
                            counts.differential += 1;
                            counts.supplemental_updates += 1;
                        }
                        if serial % 8000 == 0 {
                            let mut extra = Counts::default();
                            assert_eq!(
                                digest,
                                fragmented(
                                    message, lengths, false, relocated,
                                    &mut extra,
                                )
                            );
                            counts.empty_equivalence += 1;
                            counts.supplemental_updates += extra.updates;
                        }
                        if i == 1002 {
                            assert_eq!(digest, checkpoint.digest);
                        }
                        digest
                    }),
                );
                result.unwrap_or_else(|error| {
                panic!(
                    "{response_file}, COUNT={count}, i={i}, pattern={pattern}, \
                     lengths_in_bits={lengths:?}: {error:?}"
                )
            })
            },
            |_, _, _| {},
        );
    }
    assert_eq!(hashes, 100_000);
    assert_eq!(intermediate_pairs, 500);
    assert_eq!(counts.updates, 6_420_314);
    assert_eq!(counts.empty_updates, 4_689);
    assert_eq!(counts.relocated_messages, 100);
    assert_eq!(counts.differential, 2_150);
    assert_eq!(counts.empty_equivalence, 13);
    assert!(counts.unaligned_boundaries * 2 > counts.boundaries);
    assert!(counts.endpoints.iter().all(|&value| value > 0));
    println!(
        "{collection}: {hashes} hashes, 100 checkpoints, \
         {intermediate_pairs} intermediate M/MDi pairs; \
         {counts:?}; elapsed={:?}",
        started.elapsed()
    );
}

#[test]
fn bit_collection() {
    verify("shabittestvectors");
}

#[test]
fn byte_collection() {
    verify("shabytetestvectors");
}

#[test]
fn partition_reconstruction_and_relocation() {
    // Independent textual MSb-first oracle, including one-hot inputs to
    // distinguish every logical position. No production packing is used.
    let patterns = patterns();
    let binary: Vec<String> =
        (0..=255).map(|byte| format!("{byte:08b}")).collect();
    for active in 0..768 {
        let mut message = [0u8; 96];
        message[active / 8] = 0x80 >> (active % 8);
        let expected: String =
            message.iter().map(|byte| format!("{byte:08b}")).collect();
        for lengths in &patterns {
            let mut reconstructed = String::new();
            let mut seen = [false; 768];
            let mut offset_in_bits = 0usize;
            for (ordinal, &len_in_bits) in lengths.iter().enumerate() {
                assert!(len_in_bits > 0);
                let end_in_bits =
                    offset_in_bits.checked_add(len_in_bits).unwrap();
                assert!(end_in_bits <= 768);
                let mut storage = [0u8; 97];
                let source_offset_in_bits = 1 + ordinal % 8;
                relocate(
                    &message,
                    offset_in_bits,
                    len_in_bits,
                    source_offset_in_bits,
                    &mut storage,
                );
                for (relative, visited) in
                    seen[offset_in_bits..end_in_bits].iter_mut().enumerate()
                {
                    assert!(!*visited);
                    *visited = true;
                    let source = source_offset_in_bits + relative;
                    let byte_text = &binary[storage[source / 8] as usize];
                    reconstructed
                        .push(byte_text.as_bytes()[source % 8] as char);
                }
                offset_in_bits = end_in_bits;
            }
            assert_eq!(offset_in_bits, 768);
            assert!(seen.into_iter().all(|value| value));
            assert_eq!(reconstructed, expected);
        }
    }
}
