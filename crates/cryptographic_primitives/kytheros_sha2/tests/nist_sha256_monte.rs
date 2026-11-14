//! SHAVS (May 21, 2014) §6.4, Figure 1, through the public API.
//! Original published bytes and evidence limits: sha256-validation-evidence.md.
//! Digest concatenation preserves serialized byte order (MSb first in bytes).

use kytheros_sha2::sha256::{BitSpan, Sha256Context};
use std::{fs, path::Path};

struct Checkpoint {
    digest: [u8; 32],
    intermediate: Vec<([u8; 96], [u8; 32])>,
}

struct Monte {
    seed: [u8; 32],
    checkpoints: Vec<Checkpoint>,
}

fn hex<const N: usize>(value: &str) -> [u8; N] {
    assert_eq!(value.len(), N * 2, "hex length");
    assert!(value.bytes().all(|b| b.is_ascii_hexdigit()));
    std::array::from_fn(|index| {
        u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap()
    })
}

// Fixed ordered grammar for the inspected files. Comments are only permitted
// in the header; no metadata or trailing records can be silently skipped.
fn parse(text: &str, intermediate: bool) -> Monte {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let mut line = lines.next().expect("missing header");
    while line.starts_with('#') {
        line = lines.next().expect("missing section");
    }
    assert_eq!(line, "[L = 32]");
    let seed = hex(lines.next().unwrap().strip_prefix("Seed = ").unwrap());
    let mut checkpoints = Vec::new();
    for count in 0..100 {
        assert_eq!(lines.next().unwrap(), format!("COUNT = {count}"));
        let mut values = Vec::new();
        if intermediate {
            for inner_index in 3..=7 {
                assert_eq!(
                    lines.next().unwrap(),
                    format!("\ti = {inner_index}")
                );
                let message =
                    hex(lines.next().unwrap().strip_prefix("\tM = ").unwrap());
                let digest = hex(lines
                    .next()
                    .unwrap()
                    .strip_prefix("\tMDi = ")
                    .unwrap());
                values.push((message, digest));
            }
        }
        let digest = hex(lines.next().unwrap().strip_prefix("MD = ").unwrap());
        checkpoints.push(Checkpoint {
            digest,
            intermediate: values,
        });
    }
    assert!(lines.next().is_none(), "additional records");
    Monte { seed, checkpoints }
}

fn load(file: &str, intermediate: bool) -> Monte {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nist")
        .join(file);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    std::panic::catch_unwind(|| parse(&text, intermediate))
        .unwrap_or_else(|_| panic!("invalid fixture: {file}"))
}

fn hash(message: &[u8; 96]) -> [u8; 32] {
    let mut context = Sha256Context::new();
    let mut workspace = [0u32; 16];
    let span = BitSpan::new(message, 0, 768).unwrap();
    context.update(span, &mut workspace).unwrap();
    context.finalize(&mut workspace)
}

// Also used with a synthetic hash to test state shifting independently.
fn outer(
    seed: [u8; 32],
    mut hash: impl FnMut(&[u8; 96]) -> [u8; 32],
    mut observe: impl FnMut(usize, &[u8; 96], &[u8; 32]),
) -> [u8; 32] {
    let mut oldest_digest = seed;
    let mut middle_digest = seed;
    let mut newest_digest = seed;
    let mut message = [0u8; 96];
    for inner_index in 3..=1002 {
        message[..32].copy_from_slice(&oldest_digest);
        message[32..64].copy_from_slice(&middle_digest);
        message[64..].copy_from_slice(&newest_digest);
        let next_digest = hash(&message);
        observe(inner_index, &message, &next_digest);
        oldest_digest = middle_digest;
        middle_digest = newest_digest;
        newest_digest = next_digest;
    }
    newest_digest
}

fn compare(file: &str, count: usize, expected: &[u8], actual: &[u8]) {
    assert_eq!(
        actual, expected,
        "{file}, COUNT = {count}\nexpected = {expected:02x?}\n\
         actual = {actual:02x?}"
    );
}

fn verify(collection: &str) {
    let response_file = format!("{collection}/SHA256Monte.rsp");
    let intermediate_file = format!("{collection}/SHA256Monte.txt");
    let response = load(&response_file, false);
    let details = load(&intermediate_file, true);
    assert_eq!(response.seed, details.seed);
    let mut seed = response.seed;
    let mut hashes = 0;
    let mut intermediate_comparisons = 0;
    for (count, checkpoint) in response.checkpoints.iter().enumerate() {
        let detail = &details.checkpoints[count];
        compare(
            &intermediate_file,
            count,
            &checkpoint.digest,
            &detail.digest,
        );
        seed = outer(seed, hash, |inner_index, message, digest| {
            hashes += 1;
            if inner_index <= 7 {
                let (expected_message, expected_digest) =
                    &detail.intermediate[inner_index - 3];
                assert_eq!(
                    message, expected_message,
                    "{intermediate_file}, COUNT={count}, i={inner_index}, M"
                );
                compare(&intermediate_file, count, expected_digest, digest);
                intermediate_comparisons += 1;
            }
        });
        compare(&response_file, count, &checkpoint.digest, &seed);
    }
    assert_eq!(hashes, 100_000);
    assert_eq!(intermediate_comparisons, 500);
    println!("{collection}: 100000 hashes, 100 checkpoints, 500 M/MDi pairs");
}

#[test]
fn official_bit_collection() {
    verify("shabittestvectors");
}

#[test]
fn official_byte_collection() {
    verify("shabytetestvectors");
}

#[test]
fn synthetic_shift_and_reset() {
    // Synthetic counter outputs; no cryptographic or NIST values here.
    for seed_value in [42, 91] {
        let mut outputs = vec![seed_value; 3];
        let mut counter = 0u16;
        let result = outer(
            [seed_value; 32],
            |_| {
                counter += 1;
                [(counter % 251) as u8; 32]
            },
            |index, message, _| {
                for slot in 0..3 {
                    assert_eq!(
                        &message[slot * 32..(slot + 1) * 32],
                        &[outputs[index - 3 + slot]; 32]
                    );
                }
                outputs.push(((index - 2) % 251) as u8);
            },
        );
        assert_eq!(result, [(1000 % 251) as u8; 32]);
    }
}

#[test]
fn parser_rejections() {
    // Synthetic grammar, deliberately unrelated to official expected values.
    let mut valid = format!("[L = 32]\nSeed = {}\n", "00".repeat(32));
    for count in 0..100 {
        valid.push_str(&format!("COUNT = {count}\nMD = {}\n", "ab".repeat(32)));
    }
    assert_eq!(parse(&valid, false).checkpoints.len(), 100);
    for bad in [
        String::new(),
        valid.replace("[L = 32]", "[L = 64]"),
        valid.replace("Seed = ", "Seed = 00"),
        valid.replace("Seed = ", "Seed = gg"),
        valid.replace("MD = ", "MD = 0"),
        valid.replace("MD = ", "MD = zz"),
        valid.replace("COUNT = 1\n", "COUNT = 0\n"),
        valid.replace("COUNT = 1\n", "COUNT = 2\n"),
        valid.replace("COUNT = 1\n", ""),
        valid.replace("COUNT = 0", "Unknown = 0"),
        valid[..valid.len() - 20].to_owned(),
        valid.split("COUNT = 99").next().unwrap().to_owned(),
        format!("{valid}Extra = 0\n"),
    ] {
        assert!(std::panic::catch_unwind(|| parse(&bad, false)).is_err());
    }
}

#[test]
fn changed_checkpoint_is_detected() {
    assert!(
        std::panic::catch_unwind(|| {
            compare("synthetic", 17, &[1; 32], &[2; 32]);
        })
        .is_err()
    );
}
