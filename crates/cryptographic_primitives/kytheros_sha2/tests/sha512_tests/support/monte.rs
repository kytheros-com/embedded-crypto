//! Shared hosted SHAVS fixture grammar and recurrence.

use kytheros_sha2::sha512::{BitSpan, Sha512Context};
use std::{fs, path::Path};

pub struct Checkpoint {
    pub digest: [u8; 64],
    pub intermediate: Vec<([u8; 192], [u8; 64])>,
}

pub struct Monte {
    pub seed: [u8; 64],
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
    assert_eq!(line, "[L = 64]");
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
        .join("tests/sha512_tests/fixtures/nist")
        .join(file);
    let bytes = fs::read(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let fingerprint = match file {
        "shabittestvectors/SHA512Monte.rsp" => concat!(
            "433fa930f8427134c4979e0621d2122c79857b4ca6641a94dd31fe1c48ba6028",
            "03018137124945e3be413f51229ff7d2b96ef2aaa88f484a4930dc1cfb9ae33a",
        ),
        "shabittestvectors/SHA512Monte.txt" => concat!(
            "e788753c745a41e73cf10db0a32d134be16043de4eb273f9396124561a3b93df",
            "d52a84439468c0087ee4d0eb465ac1921adc50ca95cd977f2eac56ef20370157",
        ),
        "shabytetestvectors/SHA512Monte.rsp" => concat!(
            "6c6979558d2670aff37e6a214305e1c9ed0c9862409ecaef5c8015e324ad07c0",
            "e297d2909701c684ed3e416add1b27603f7287d413d29753b2882e6e803f142b",
        ),
        "shabytetestvectors/SHA512Monte.txt" => concat!(
            "9892589552dc5867cf2a579a4c1a7eb81ef075656ba86809b69632bc54ae298d",
            "2fa430930bd92ec789de939619ec1826ded7bf169bc4a83e3ed94eef7c4fe4c0",
        ),
        _ => panic!("unexpected Monte Carlo fixture: {file}"),
    };
    assert_eq!(
        hash_bits(&bytes, bytes.len() * 8),
        hex::<64>(fingerprint),
        "fixture integrity: {file}"
    );
    let text = std::str::from_utf8(&bytes).expect(file);
    std::panic::catch_unwind(|| parse(text, intermediate))
        .unwrap_or_else(|_| panic!("invalid fixture: {file}"))
}

pub fn hash(message: &[u8; 192]) -> [u8; 64] {
    hash_bits(message, 1536)
}

fn hash_bits(message: &[u8], len_in_bits: usize) -> [u8; 64] {
    let mut context = Sha512Context::new();
    let mut workspace = [0u64; 16];
    let span = BitSpan::new(message, 0, len_in_bits).unwrap();
    context.update(span, &mut workspace).unwrap();
    context.finalize(&mut workspace)
}

// Also used with a synthetic hash to test state shifting independently.
pub fn outer(
    seed: [u8; 64],
    mut hash: impl FnMut(usize, &[u8; 192]) -> [u8; 64],
    mut observe: impl FnMut(usize, &[u8; 192], &[u8; 64]),
) -> [u8; 64] {
    let mut oldest_digest = seed;
    let mut middle_digest = seed;
    let mut newest_digest = seed;
    let mut message = [0u8; 192];
    for inner_index in 3..=1002 {
        message[..64].copy_from_slice(&oldest_digest);
        message[64..128].copy_from_slice(&middle_digest);
        message[128..].copy_from_slice(&newest_digest);
        let next_digest = hash(inner_index, &message);
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
