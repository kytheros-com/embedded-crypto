//! SHAVS (May 21, 2014) §6.4, Figure 1, through the public API.
//! Original published bytes and evidence limits: sha256-validation-evidence.md.
//! Digest concatenation preserves serialized byte order (MSb first in bytes).

#[path = "support/monte.rs"]
mod monte;
use monte::*;

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
