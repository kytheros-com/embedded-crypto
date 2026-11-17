//! Shared hosted SHAVS fixture grammar and recurrence.

use kytheros_sha2::sha256::{BitSpan, Sha256Context};
use std::{fs, path::Path};

pub struct Checkpoint {
    pub digest: [u8; 32],
    pub intermediate: Vec<([u8; 96], [u8; 32])>,
}

pub struct Monte {
    pub seed: [u8; 32],
    pub checkpoints: Vec<Checkpoint>,
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
pub fn parse(text: &str, intermediate: bool) -> Monte {
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

pub fn load(file: &str, intermediate: bool) -> Monte {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sha256_tests/fixtures/nist")
        .join(file);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    std::panic::catch_unwind(|| parse(&text, intermediate))
        .unwrap_or_else(|_| panic!("invalid fixture: {file}"))
}

pub fn hash(message: &[u8; 96]) -> [u8; 32] {
    let mut context = Sha256Context::new();
    let mut workspace = [0u32; 16];
    let span = BitSpan::new(message, 0, 768).unwrap();
    context.update(span, &mut workspace).unwrap();
    context.finalize(&mut workspace)
}

// Also used with a synthetic hash to test state shifting independently.
pub fn outer(
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

pub fn compare(file: &str, count: usize, expected: &[u8], actual: &[u8]) {
    assert_eq!(
        actual, expected,
        "{file}, COUNT = {count}\nexpected = {expected:02x?}\n\
         actual = {actual:02x?}"
    );
}
