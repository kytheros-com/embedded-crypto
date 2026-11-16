//! Published CAVP ordinary-message evidence through the public SHA-256 API.
//! See docs/assurance/sha256-validation-evidence.md for provenance and limits.
//! Hosted-only fixtures are read at runtime; firmware contains none of them.

use kytheros_sha2::sha256::{BitSpan, Sha256Context};
use std::{fs, path::Path};

struct Case {
    len_in_bits: usize,
    message: Vec<u8>,
    expected: Vec<u8>,
}

fn hex(value: &str) -> Vec<u8> {
    assert!(!value.is_empty(), "empty hexadecimal field");
    assert_eq!(value.len() % 2, 0, "odd hexadecimal field");
    assert!(value.bytes().all(|b| b.is_ascii_hexdigit()));
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
        })
        .collect()
}

// Deliberately accepts only the ordinary SHA-256 grammar present in these
// archives. Unexpected metadata, duplicate fields and incomplete records
// are errors, including an EOF after a complete but shortened case list.
fn parse(text: &str, expected_count: usize) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut section = false;
    let mut length = None;
    let mut message = None;
    let mut boundary = true;
    for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        let location = format!("line {}", line_index + 1);
        if line.is_empty() {
            assert!(length.is_none(), "incomplete record at {location}");
            boundary = true;
            continue;
        }
        if line.starts_with('#') {
            assert!(length.is_none(), "comment inside record at {location}");
            continue;
        }
        if line == "[L = 32]" {
            assert!(!section && cases.is_empty() && length.is_none());
            section = true;
            continue;
        }
        assert!(section, "missing SHA-256 section at {location}");
        let (key, value) = line.split_once(" = ").expect(&location);
        match key {
            "Len" => {
                assert!(boundary && length.is_none(), "{location}");
                assert!(!value.is_empty());
                assert!(value.bytes().all(|b| b.is_ascii_digit()));
                length = Some(value.parse::<usize>().expect(&location));
                boundary = false;
            }
            "Msg" => {
                let len_in_bits = length.expect(&location);
                assert!(message.is_none(), "duplicate Msg at {location}");
                let bytes = hex(value);
                if len_in_bits == 0 {
                    assert_eq!(bytes, [0], "empty Msg placeholder");
                } else {
                    assert_eq!(bytes.len(), len_in_bits.div_ceil(8));
                    let unused_in_bits = (8 - len_in_bits % 8) % 8;
                    assert_eq!(
                        bytes.last().unwrap()
                            & ((1u16 << unused_in_bits) - 1) as u8,
                        0
                    );
                }
                message = Some(bytes);
            }
            "MD" => {
                let len_in_bits = length.take().expect(&location);
                let message = message.take().expect(&location);
                let expected = hex(value);
                assert_eq!(expected.len(), 32, "{location}");
                cases.push(Case {
                    len_in_bits,
                    message,
                    expected,
                });
            }
            _ => panic!("unexpected field at {location}: {key}"),
        }
    }
    assert!(section && length.is_none(), "empty or incomplete file");
    assert!(expected_count > 0);
    assert_eq!(cases.len(), expected_count, "fixture case count");
    cases
}

fn digest(message: &[u8], len_in_bits: usize) -> [u8; 32] {
    let mut context = Sha256Context::new();
    let mut workspace = [0; 16];
    let input = BitSpan::new(message, 0, len_in_bits).unwrap();
    context.update(input, &mut workspace).unwrap();
    context.finalize(&mut workspace)
}

fn load(file: &str, count: usize, checksum: &str) -> Vec<Case> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/nist")
        .join(file);
    let bytes = fs::read(&path).unwrap_or_else(|e| {
        panic!("required NIST fixture {}: {e}", path.display())
    });
    // This is an integrity regression check, not an authenticity oracle.
    assert_eq!(
        digest(&bytes, bytes.len() * 8).as_slice(),
        hex(checksum),
        "fixture integrity: {file}"
    );
    let text = std::str::from_utf8(&bytes).expect(file);
    std::panic::catch_unwind(|| parse(text, count))
        .unwrap_or_else(|_| panic!("invalid NIST fixture: {file}"))
}

fn verify(file: &str, cases: &[Case], incremental: bool) {
    for (index, case) in cases.iter().enumerate() {
        let actual = if incremental {
            let mut context = Sha256Context::new();
            let mut workspace = [0; 16];
            let mut offset_in_bits = 0;
            // Repeatedly crosses byte, padding and block boundaries.
            for step_in_bits in
                [1, 7, 8, 55, 448, 511, 512, 513].into_iter().cycle()
            {
                let len_in_bits =
                    step_in_bits.min(case.len_in_bits - offset_in_bits);
                let span =
                    BitSpan::new(&case.message, offset_in_bits, len_in_bits)
                        .unwrap();
                context.update(span, &mut workspace).unwrap();
                offset_in_bits += len_in_bits;
                if offset_in_bits == case.len_in_bits {
                    break;
                }
            }
            context.finalize(&mut workspace)
        } else {
            digest(&case.message, case.len_in_bits)
        };
        assert_eq!(
            actual.as_slice(),
            case.expected,
            "{file}, case {} (1-based), Len = {}, incremental = {}\n\
             expected = {:02x?}\nactual = {:02x?}",
            index + 1,
            case.len_in_bits,
            incremental,
            case.expected,
            actual
        );
    }
    println!(
        "{file}: discovered {}, executed {} (incremental={})",
        cases.len(),
        cases.len(),
        incremental
    );
}

macro_rules! fixture_test {
    ($name:ident, $file:literal, $count:literal, $checksum:literal) => {
        #[test]
        fn $name() {
            let cases = load($file, $count, $checksum);
            verify($file, &cases, false);
        }
    };
}

fixture_test!(
    bit_short,
    "shabittestvectors/SHA256ShortMsg.rsp",
    513,
    "3993d9bb135b37f9226420032630f0e34d9b20556160baa9d85546cc421f183e"
);
fixture_test!(
    bit_long,
    "shabittestvectors/SHA256LongMsg.rsp",
    512,
    "a6228c355f9c35faf37658d1290fa9044b252204056b4db56eb1d5a623be9897"
);
fixture_test!(
    byte_short,
    "shabytetestvectors/SHA256ShortMsg.rsp",
    65,
    "75e1cb83994638481808e225b9eb0c1ebd0c232d952ac42b61abce6363be283c"
);
fixture_test!(
    byte_long,
    "shabytetestvectors/SHA256LongMsg.rsp",
    64,
    "6fac36f37360bcf74ffcf4465c18e30d6d5a04cc90885b901fc3130c16060974"
);

#[test]
fn incremental_official_bit_messages() {
    let file = "shabittestvectors/SHA256ShortMsg.rsp";
    let cases = load(
        file,
        513,
        "3993d9bb135b37f9226420032630f0e34d9b20556160baa9d85546cc421f183e",
    );
    verify(file, &cases, true);
    let file = "shabittestvectors/SHA256LongMsg.rsp";
    let cases = load(
        file,
        512,
        "a6228c355f9c35faf37658d1290fa9044b252204056b4db56eb1d5a623be9897",
    );
    verify(file, &cases, true);
}

#[test]
fn parser_rejects_malformed_and_truncated_records() {
    // Synthetic grammar fixtures only; these are not NIST test vectors.
    let valid = format!(
        "# comment\n[L = 32]\n\nLen = 0\nMsg = 00\nMD = {}\n",
        "00".repeat(32)
    );
    assert_eq!(parse(&valid, 1)[0].len_in_bits, 0);
    for bad in [
        String::new(),
        valid.replace("[L = 32]", "[L = 64]"),
        valid.replace("Len = 0", "Len = -1"),
        valid.replace("Msg = 00", "Msg = gg"),
        valid.replace("Msg = 00", "Msg = 0"),
        valid.replace("Msg = 00", "Msg = 01"),
        valid.replace("Msg = 00", "Msg = 00\nMsg = 00"),
        valid.replace("Msg = 00", "Unknown = 00"),
        valid.replace("Len = 0", "Len = 9"),
        valid
            .replace("Len = 0", "Len = 1")
            .replace("Msg = 00", "Msg = 01"),
        valid.replace("MD = ", "MD = 0"),
        valid.split("MD =").next().unwrap().to_owned(),
        format!("{valid}\nLen = 0\n"),
    ] {
        assert!(std::panic::catch_unwind(|| parse(&bad, 1)).is_err());
    }
    assert!(std::panic::catch_unwind(|| parse(&valid, 2)).is_err());
}
