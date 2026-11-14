# SHA-256 published-vector verification methodology

**Status:** Internal development methodology; subject to human review.

## Scope and references

The implementation is `kytheros_sha2` 0.1.0, portable safe Rust SHA-256,
`src/sha256/`, using the exported `Sha256Context` and `BitSpan` APIs.
The algorithm reference is [FIPS 180-4 (August 2015)](https://doi.org/10.6028/NIST.FIPS.180-4),
§§3.1–3.2 (representation), §5.1.1 (padding), §5.2.1 (blocks),
§5.3.3 (initialization), and §6.2.2 (hash computation).

This methodology follows [the documentation standard](../documentation-standard.md),
[architecture](architecture.md), [project scope](project-scope.md),
[MCL FSM](../design/mcl-fsm.md), and
[module interfaces](../design/module-interfaces.md).
The architecture's CMVP workbook remains the controlling assurance planning
baseline; this evidence does not change requirements, mappings, decisions,
MCL state ownership, or PIL responsibilities. No specification conflict was
identified during this integration.

## Sources and provenance

NIST publishes these collections on its
[CAVP Secure Hashing page](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/secure-hashing):

- [shabittestvectors.zip](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabittestvectors.zip)
- [shabytetestvectors.zip](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabytetestvectors.zip)

Both original archives were already tracked under
`crates/cryptographic_primitives/kytheros_sha2/src/sha256/NIST_KAT_Files/`.
The user identified these supplied archives on 2026-11-14. Their original
network acquisition date and independent authenticity verification are
unknown. A sandbox download attempt failed DNS resolution; no fresh download
was used. Original contents were available locally and inspected.

The machine-readable
[provenance manifest](../../crates/cryptographic_primitives/kytheros_sha2/tests/fixtures/nist/provenance.json)
records official URLs, exact archive filenames, SHA-256 integrity checksums,
receipt date, internal ZIP paths, extracted-file checksums and case counts.
These checksums identify observed bytes; they are not NIST signatures or
independent proof of authenticity. Human review should verify source
provenance separately if stronger authenticity evidence is required.

Archive integrity checksums:

| Archive | SHA-256 |
| --- | --- |
| `shabittestvectors.zip` | `cd7b9f11680c6e0ccdbe13b28403f2017b5ff48789152162461e0a24fb4c5d45` |
| `shabytetestvectors.zip` | `929ef80b7b3418aca026643f6f248815913b60e01741a44bba9e118067f4c9b8` |

Extracted fixtures reside under `kytheros_sha2/tests/fixtures/nist/`, using
unchanged archive-internal paths. Both `Readme.txt` files are preserved in
full, including original line endings. Message and digest values are not
rewritten. Each README has SHA-256
`24bf370543f9521f7eb44f49e8d489ac9e45a21126f883d8e32e7fffd1d10227`.

| Internal response path | SHA-256 | Cases |
| --- | --- | ---: |
| `shabittestvectors/SHA256ShortMsg.rsp` | `3993d9bb135b37f9226420032630f0e34d9b20556160baa9d85546cc421f183e` | 513 |
| `shabittestvectors/SHA256LongMsg.rsp` | `a6228c355f9c35faf37658d1290fa9044b252204056b4db56eb1d5a623be9897` | 512 |
| `shabytetestvectors/SHA256ShortMsg.rsp` | `75e1cb83994638481808e225b9eb0c1ebd0c232d952ac42b61abce6363be283c` | 65 |
| `shabytetestvectors/SHA256LongMsg.rsp` | `6fac36f37360bcf74ffcf4465c18e30d6d5a04cc90885b901fc3130c16060974` | 64 |

## Harness and interpretation

[The integration harness](../../crates/cryptographic_primitives/kytheros_sha2/tests/nist_sha256.rs)
reads the four retained response files at runtime. Each ordinary case
constructs `Sha256Context`, validates `BitSpan`, supplies sixteen `u32`
schedule words, calls `update`, then consuming `finalize`, and compares
all 32 output bytes. Failure diagnostics identify file, one-based case
index, bit length, expected digest and actual digest.

The grammar is deliberately restricted to what these files contain:
comment lines, blank record separators, one `[L = 32]` section, then
ordered `Len`, `Msg`, `MD` fields. `Len` is unsigned decimal bits;
`Msg` and `MD` are complete hexadecimal byte strings. Unknown fields,
metadata, duplicate or out-of-order fields, invalid hex, wrong lengths,
incomplete records and incorrect case counts fail. Exact fixture checksums
also detect alterations and truncation after an otherwise complete record.
Missing fixtures fail rather than skip. Synthetic parser-negative inputs
are local harness tests, not published NIST vectors.

The READMEs describe response files and distinguish Monte Carlo intermediate
values; they do not explicitly define partial-byte packing or the zero-length
placeholder. The response files show `Msg = 00` for `Len = 0`; only `Len`
meaningful bits are supplied, so that placeholder contributes zero bits.
For positive lengths, storage contains exactly `ceil(Len/8)` bytes.
Meaningful bits occupy the most significant positions of the final byte;
unused low bits are zero in the retained files, which the parser checks.
This interpretation follows FIPS 180-4 §3.1 and the existing MSb-first
`BitSpan` contract; it is not attributed to an absent README statement.
Digest bytes are in big-endian H_0 through H_7 order (§6.2.2).

[SHAVS (updated May 21, 2014)](https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/SHAVS.pdf)
§4.2 defines `Len` in bits and `L` in bytes. Sections 6.2 and 6.3 describe
ordinary ShortMsg and selected LongMsg categories. All 1,154 retained
ordinary cases execute without sampling or arbitrary limits.

A separate incremental test replays all 1,025 bit-oriented messages using
a repeating update-size sequence of 1, 7, 8, 55, 448, 511, 512 and 513 bits,
clipped only at the message endpoint. Expected digests remain the published
NIST values. Empty input includes an empty update. This covers varying source
alignments and block crossings but not every possible partition.

Existing local exhaustive padding tests, synthetic near-limit tests, and
`context/reference_vectors.py` remain separate development evidence and are
preserved. They are not relabeled as published NIST data.

## Monte Carlo procedure and evidence

The separate hosted integration target `tests/nist_sha256_monte.rs` implements
SHAVS (updated May 21, 2014) §6.4, Figure 1. For each COUNT j = 0..99,
MD_0 = MD_1 = MD_2 = Seed. For i = 3..1002 inclusive, it hashes
M_i = MD_(i-3) || MD_(i-2) || MD_(i-1), shifts the three digests, compares
MD_1002 with the response checkpoint, and sets Seed = MD_1002. The next
outer iteration resets all three digests to that seed. Each message is
96 bytes (768 bits), preserving serialized digest byte order, MSb first
within each byte; SHA-256 digests serialize H_0 through H_7 big-endian.
Each collection executes exactly 100,000 hashes and 100 primary comparisons.
Every hash uses new(), BitSpan::new(), update(), and consuming finalize(),
with a caller-owned 16-word workspace and one fixed 96-byte message array.
There are no allocations in the hashing loop.

The original bit and byte collections differ in bytes, checksums, seeds,
and checkpoints; both execute, totaling 200,000 hashes and 200 primary
checkpoint comparisons per complete Rust run. Neither substitutes for the
other. SHAVS §6 states that MCT itself always uses byte-oriented messages.
The READMEs explicitly define the intermediate indices as i = 3..7,
without an offset. For every COUNT, both the published M and MDi are
checked: 500 message and 500 digest comparisons per collection. Final MD
in each intermediate file must also equal the corresponding response MD.
There is no intermediate-index ambiguity in these sources.

The strict parser requires one [L = 32], a 32-byte Seed, ordered COUNT
0..99 and corresponding 32-byte MD fields; the detailed grammar additionally
requires the five ordered tab-indented i/M/MDi groups, with 96-byte M.
Header comments and blank lines are allowed; unknown records, malformed
hex, missing or duplicated fields, wrong lengths, out-of-order counts,
truncation and extra records fail. Missing fixtures fail. Synthetic parser,
rolling-state and mismatch-detection tests are local tests, not NIST vectors.

All four files are extracted unchanged with original CRLF; Git attributes
preserve these bytes. The manifest records each checksum and archive path.

| Collection/file | SHA-256 |
| --- | --- |
| `shabittestvectors/SHA256Monte.rsp` | `a0dd9e7cb6ac6497360eca29a913327466c4c23763e6a2a1c43eb27d63ea72ac` |
| `shabittestvectors/SHA256Monte.txt` | `192c0b77bd699a8809c0617c66714a7ff59920683628696edf29a525bc8b581c` |
| `shabytetestvectors/SHA256Monte.rsp` | `29ea30c6bb4b84e425fb8c1d731c6bb852dac935825f2bd1143e5d3c4f10bfb9` |
| `shabytetestvectors/SHA256Monte.txt` | `0553303fe5b95ee2a5d8a1671e9bd94f403475e1a2f4ca43f59575cb39d03b3a` |

`tests/fixtures/nist/verify_monte.py` separately computes the recurrence
using Python 3 hashlib and a digest history list, independently of Rust's
rolling triple. It compares all primary checkpoints and all first-five
M/MDi pairs for both collections. These computed results are reference
results, distinct from both official expectations and Rust results.
`verify.py` checks exact extraction equality and all retained checksums;
checksums establish local integrity, not independent NIST authentication.

## Reproduction and CI

From the repository root:

```sh
python3 crates/cryptographic_primitives/kytheros_sha2/tests/fixtures/nist/verify.py
cargo test -p kytheros_sha2 --test nist_sha256 --locked --offline -- --nocapture
python3 crates/cryptographic_primitives/kytheros_sha2/tests/fixtures/nist/verify_monte.py
cargo test -p kytheros_sha2 --test nist_sha256_monte --locked --offline
cargo test --workspace --all-targets --locked --offline
```

`verify.py` uses Python's `hashlib` and `zipfile` to check both archive
checksums, every retained file checksum, exact extraction equality and
record counts without invoking the Rust implementation. The Cargo harness
also checks response-file checksums through the public Rust API as an
integrity regression check; that check is not an independent hash oracle.

Cargo automatically discovers the integration target, so the existing CI
workspace test command executes it. No workflow change, test-time download,
new dependency, firmware fixture inclusion or ignored test is needed.
Rustfmt, Clippy, Rustdoc, line-length and embedded-target checks remain
required. The original archives remain in place; extraction is unnecessary
at test time. Particular outcomes and revision/environment identification
belong in [execution records](sha256-validation-runs/2026-11-14.md), and [Monte Carlo results](sha256-validation-runs/2026-11-14-monte.md), not in
this methodology as a permanent passing claim.

## Coverage limits and review

These published NIST CAVP test vectors support internal algorithm-correctness
verification only. Passing does not constitute official CAVP validation,
ACVTS certification, FIPS 140-3 module validation, CMVP certification,
independent security evaluation, side-channel resistance, secure zeroization,
or a complete module cryptographic self-test design. Hosted execution is
not execution on embedded hardware. Human review must assess provenance,
parser interpretation, evidence sufficiency and integration changes.
AI-generated work remains unapproved until accepted through project review.
