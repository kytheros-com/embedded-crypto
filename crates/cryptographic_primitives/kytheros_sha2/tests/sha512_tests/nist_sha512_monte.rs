//! SHAVS (May 21, 2014) §6.4, Figure 1, through the public API.
//! Original published bytes and evidence limits: sha512-validation-evidence.md.
//! Digest concatenation preserves serialized byte order (MSb first in bytes).

#[path = "support/monte.rs"]
mod monte;
use monte::*;

fn verify(collection: &str) {
    let response_file = format!("{collection}/SHA512Monte.rsp");
    let intermediate_file = format!("{collection}/SHA512Monte.txt");
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
        seed = outer(
            seed,
            |_, message| hash(message),
            |inner_index, message, digest| {
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
            },
        );
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
    // Distinct byte and digest sequences expose both byte reversal and
    // rolling-window order errors. The second outer loop uses the first
    // loop's actual result to regenerate all three initial slots.
    let synthetic =
        |tag: u16| std::array::from_fn(|i| tag.to_be_bytes()[i % 2] ^ i as u8);
    let mut seed = synthetic(0x0123);
    for outer_index in 0..2u16 {
        let mut history = vec![seed; 3];
        seed = outer(
            seed,
            |inner_index, _| synthetic(outer_index * 2048 + inner_index as u16),
            |index, message, digest| {
                assert_eq!(index, history.len());
                for (slot, bytes) in
                    message.as_chunks::<64>().0.iter().enumerate()
                {
                    assert_eq!(*bytes, history[index - 3 + slot]);
                }
                history.push(*digest);
            },
        );
        assert_eq!(history.len(), 1003);
        assert_eq!(seed, synthetic(outer_index * 2048 + 1002));
    }
}

#[test]
fn parser_rejections() {
    // Synthetic grammar, deliberately unrelated to official expected values.
    let mut valid = format!("[L = 64]\nSeed = {}\n", "00".repeat(64));
    for count in 0..100 {
        valid.push_str(&format!("COUNT = {count}\nMD = {}\n", "ab".repeat(64)));
    }
    assert_eq!(parse(&valid, false).checkpoints.len(), 100);
    let mut detailed = valid.clone();
    let groups: String = (3..=7)
        .map(|i| {
            format!(
                "\ti = {i}\n\tM = {}\n\tMDi = {}\n",
                "12".repeat(192),
                "34".repeat(64)
            )
        })
        .collect();
    detailed = detailed.replace("MD = ", &(groups + "MD = "));
    assert_eq!(parse(&detailed, true).checkpoints[0].intermediate.len(), 5);
    for bad in [
        detailed.replace("\ti = 3", "\ti = 8"),
        detailed.replace("\tM = ", "M = "),
        detailed.replace("\tMDi = ", "\tMDi = 0"),
        detailed.replace("\tM = ", "\tM = ff"),
    ] {
        assert!(std::panic::catch_unwind(|| parse(&bad, true)).is_err());
    }
    for bad in [
        String::new(),
        valid.replace("[L = 64]", "[L = 65]"),
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
            compare("synthetic", 17, &[1; 64], &[2; 64]);
        })
        .is_err()
    );
}
