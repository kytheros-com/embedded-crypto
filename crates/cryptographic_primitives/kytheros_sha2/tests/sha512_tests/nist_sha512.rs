//! Published CAVP ordinary-message evidence through the public SHA-512 API.
//! See docs/assurance/sha512-validation-evidence.md for provenance and limits.
//! Hosted-only fixtures are read at runtime; firmware contains none of them.

use kytheros_sha2::sha512::{BitSpan, Sha512Context};
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

// Deliberately accepts only the ordinary SHA-512 grammar present in these
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
            assert!(!section, "comment after section at {location}");
            continue;
        }
        if line == "[L = 64]" {
            assert!(!section && cases.is_empty() && length.is_none());
            section = true;
            continue;
        }
        assert!(section, "missing SHA-512 section at {location}");
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
                assert_eq!(expected.len(), 64, "{location}");
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

fn digest(message: &[u8], len_in_bits: usize) -> [u8; 64] {
    let mut context = Sha512Context::new();
    let mut workspace = [0; 16];
    let input = BitSpan::new(message, 0, len_in_bits).unwrap();
    context.update(input, &mut workspace).unwrap();
    context.finalize(&mut workspace)
}

fn load(file: &str, count: usize, checksum: &str) -> Vec<Case> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sha512_tests/fixtures/nist")
        .join(file);
    let bytes = fs::read(&path).unwrap_or_else(|e| {
        panic!("required NIST fixture {}: {e}", path.display())
    });
    // Python hashlib supplied this file fingerprint for regression checking.
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
            let mut context = Sha512Context::new();
            let mut workspace = [0; 16];
            let mut offset_in_bits = 0;
            // Repeatedly crosses byte, padding and block boundaries.
            for step_in_bits in [1, 7, 8, 55, 895, 896, 1023, 1024, 1025]
                .into_iter()
                .cycle()
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

struct Fixture {
    path: &'static str,
    count: usize,
    fingerprint: &'static str,
    length: fn(usize) -> usize,
}

impl Fixture {
    fn verify(&self, incremental: bool) {
        let cases = load(self.path, self.count, self.fingerprint);
        for (index, case) in cases.iter().enumerate() {
            assert_eq!(
                case.len_in_bits,
                (self.length)(index),
                "{}: unexpected length for case {}",
                self.path,
                index + 1
            );
        }
        verify(self.path, &cases, incremental);
    }
}

const BIT_SHORT: Fixture = Fixture {
    path: "shabittestvectors/SHA512ShortMsg.rsp",
    count: 1025,
    fingerprint: concat!(
        "48dca842ab35d132e8f047950768252340103f62d755cef9becd89c296173a92",
        "a877bf8c834e91a1e446f2aea568198c43ae2462def4b5bb9b565cc774773459",
    ),
    length: |index| index,
};

#[test]
fn bit_short() {
    BIT_SHORT.verify(false);
}

const BIT_LONG: Fixture = Fixture {
    path: "shabittestvectors/SHA512LongMsg.rsp",
    count: 1024,
    fingerprint: concat!(
        "ff3e8d41cb37a7541f81e6bcb775f67b9fcc7263017e41ee8800f3f574bd39dd",
        "343693593e123c8735be7fbb78938bf38f588ab897d2fab4f66054cdd82f64d9",
    ),
    length: |index| 1024 + 99 * (index + 1),
};

#[test]
fn bit_long() {
    BIT_LONG.verify(false);
}

const BYTE_SHORT: Fixture = Fixture {
    path: "shabytetestvectors/SHA512ShortMsg.rsp",
    count: 129,
    fingerprint: concat!(
        "0d7b05af31f39db8cfe13f7f78f07e33a729189bb951be3c4e5fc00e192373bf",
        "45b082805ca06e7c455cb8e295b5d947e2096fc75eb002a8ed4dd18f6b35d58c",
    ),
    length: |index| 8 * index,
};

#[test]
fn byte_short() {
    BYTE_SHORT.verify(false);
}

const BYTE_LONG: Fixture = Fixture {
    path: "shabytetestvectors/SHA512LongMsg.rsp",
    count: 128,
    fingerprint: concat!(
        "5ae8696e7250805a9deec01308348304817ac4851dd0edf577682461c64dc14b",
        "4491aef097ad5e476c836b19c367ed13bbe0a91787afdd284e20c9aabf2bce50",
    ),
    length: |index| 1024 + 792 * (index + 1),
};

#[test]
fn byte_long() {
    BYTE_LONG.verify(false);
}

#[test]
fn incremental_official_bit_messages() {
    BIT_SHORT.verify(true);
    BIT_LONG.verify(true);
}

#[test]
fn parser_rejects_malformed_and_truncated_records() {
    // Synthetic grammar fixtures only; these are not NIST test vectors.
    let valid = format!(
        "# comment\n[L = 64]\n\nLen = 0\nMsg = 00\nMD = {}\n",
        "00".repeat(64)
    );
    assert_eq!(parse(&valid, 1)[0].len_in_bits, 0);
    for bad in [
        String::new(),
        valid.replace("[L = 64]", "[L = 65]"),
        valid.replace("Len = 0", "Len = -1"),
        valid.replace("Msg = 00", "Msg = gg"),
        valid.replace("Msg = 00", "Msg = 0"),
        valid.replace("Msg = 00", "Msg = 01"),
        valid.replace("Msg = 00", "Msg = 00\nMsg = 00"),
        valid.replace("Msg = 00", "Unknown = 00"),
        valid.replace("Len = 0", "Len = +0"),
        valid.replace("Len = 0", "Len = 0\nLen = 0"),
        valid.replace("Len = 0\nMsg = 00", "Msg = 00\nLen = 0"),
        valid.replace("MD = ", "MD = xx"),
        valid.replace("[L = 64]", "[L = 64]\n[L = 64]"),
        valid.replace("Msg = 00", "Msg = 00\n# late comment"),
        valid.replace("Len = 0", "Len = 9"),
        valid
            .replace("Len = 0", "Len = 1")
            .replace("Msg = 00", "Msg = 01"),
        valid.replace("MD = ", "MD = 0"),
        valid.split("MD =").next().unwrap().to_owned(),
        format!("{valid}\nLen = 0\n"),
        format!("{valid}\n{}", valid.split("\n\n").nth(1).unwrap()),
    ] {
        assert!(std::panic::catch_unwind(|| parse(&bad, 1)).is_err());
    }
    assert!(std::panic::catch_unwind(|| parse(&valid, 2)).is_err());
}
