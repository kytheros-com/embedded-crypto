# SHA-256 published-vector verification methodology

**Status:** Internal development methodology; subject to human review.

## Scope and references

The implementation is `kytheros_sha2` 0.1.0, portable safe Rust SHA-256,
`src/sha256.rs` and `src/sha256/`, using the exported `Sha256Context` and
`BitSpan` APIs.
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
`crates/cryptographic_primitives/kytheros_sha2/tests/NIST_KAT_Files/`.
The user identified these supplied archives on 2026-11-14. Their original
network acquisition date and independent authenticity verification are
unknown. A sandbox download attempt failed DNS resolution; no fresh download
was used. Original contents were available locally and inspected.

The machine-readable
[provenance manifest](../../crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/fixtures/nist/provenance.json)
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

Extracted fixtures reside under
`kytheros_sha2/tests/sha256_tests/fixtures/nist/`, using
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

[The integration harness](../../crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/nist_sha256.rs)
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
`tests/sha256_tests/unit_tests/context_reference_vectors.py` remain separate
development evidence and are
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

`tests/sha256_tests/fixtures/nist/verify_monte.py` separately computes the
recurrence
using Python 3 hashlib and a digest history list, independently of Rust's
rolling triple. It compares all primary checkpoints and all first-five
M/MDi pairs for both collections. These computed results are reference
results, distinct from both official expectations and Rust results.
`verify.py` checks exact extraction equality and all retained checksums;
checksums establish local integrity, not independent NIST authentication.

## Reproduction and CI

From the repository root:

```sh
python3 crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/fixtures/nist/verify.py
cargo test -p kytheros_sha2 --test nist_sha256 --locked --offline -- --nocapture
python3 crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/fixtures/nist/verify_monte.py
cargo test -p kytheros_sha2 --test nist_sha256_monte --locked --offline
cargo test --workspace --all-targets --locked --offline
```

`verify.py` uses Python's `hashlib` and `zipfile` to check both archive
checksums, every retained file checksum, exact extraction equality and
record counts without invoking the Rust implementation. The Cargo harness
also checks response-file checksums through the public Rust API as an
integrity regression check; that check is not an independent hash oracle.

Integration tests are explicitly declared in the crate's Cargo.toml file,
so the existing CI workspace test command executes it. No workflow change,
test-time download, new dependency, firmware fixture inclusion or ignored
test is needed.
Rustfmt, Clippy, Rustdoc, line-length and embedded-target checks remain
required. The original archives are located at
`crates/cryptographic_primitives/kytheros_sha2/tests/NIST_KAT_Files`;
extraction is unnecessary
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

## Kytheros fragmented-update Monte Carlo stress testing

This additional procedure is designed by Kytheros; NIST does not prescribe
this partitioning. Official NIST seeds, intermediate values and final
checkpoints remain the expectations. The shared SHAVS §6.4 recurrence is
unchanged. The test exercises the public incremental API and arbitrary-bit
packing with MSb-first source fragments, in serialized digest byte order.
This is functional correctness evidence, not formal certification.

The hosted integration target
[`sha256_fragmented_monte.rs`](../../crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/sha256_fragmented_monte.rs)
uses the parser, single-update hash and recurrence extracted without
behavior changes into
[`support/monte.rs`](../../crates/cryptographic_primitives/kytheros_sha2/tests/sha256_tests/support/monte.rs).
Existing NIST parser, shifting and mismatch tests still run in the original
target. Production source, ordinary-message tests and all fixture bytes and
expectations remain unchanged. Cargo automatically discovers the new target;
no feature, dependency, workflow change or firmware fixture inclusion is needed.

### Deterministic partition schedule

Let `n = COUNT * 1000 + i - 3` (0 through 99,999 per collection), and select
slot `n % 32`. The 32 precomputed slots are:

| Slots | Nonempty fragment lengths in bits |
| --- | --- |
| 0 | 768 fragments of one bit |
| 1 | 256 fragments of three bits |
| 2 | Cyclic irregular list, rotation zero, clipped at message end |
| 3 | 13, 499, 1, 255 (endpoint 512 after two updates) |
| 4–11 | 1+767, 767+1, 447+321, 448+320, 449+319, 511+257, 512+256, 513+255 |
| 12–31 | Irregular rotations `(slot - 12) % 9` |

The irregular list is `[1,3,5,7,9,13,17,31,63]`, repeated cyclically from
the selected rotation until exactly 768 bits are consumed; the final length
is clipped to the remaining bits. Repeated rotations deliberately weight
small odd-length fragments. Every slot occurs 3,125 times per collection.
Each message has at least two nonempty updates. Fragment endpoint addition
is checked, lengths must be positive, no endpoint may exceed 768, and the
final endpoint must equal 768. The ordinary path uses `BitSpan` directly at
the accumulated logical offset into the original 96-byte message.

For `n % 64 == 0`, empty updates are inserted before the message, after the
first fragment, and after the final fragment at offset 768. All 1,563 such
messages per collection are directly compared with a single-update digest.
For `n % 8000 == 0`, 13 of those messages are also replayed through the same
fragmentation and relocation settings with empty updates removed.

For `n % 1000 == 0`, all fragments of the message are separately relocated
into a caller-owned 97-byte temporary array. Fragment ordinal k is placed at
source offset `1 + k % 8`, covering offsets 1 through 8 with MSb-first bit
copies and nonzero outside-span sentinels. There are 100 relocated recurrence
messages per collection, plus 13 relocated empty-free replays. No production
`pack_bits` is used for relocation or as a reference oracle.

### Harness verification and differential sampling

Every one of the 32 partition slots is independently reconstructed on all
768 one-hot 96-byte inputs (24,576 cases). The test tracks each source
position exactly once, verifies positive lengths, order, checked endpoints,
complete consumption and final offset 768. Relocated fragments are decoded
through independently formatted eight-character binary byte strings and
concatenated for exact comparison with the original MSb-first message text.
This exhausts the finite partition schedule and all one-hot positions,
including source and logical destination alignment. It is separate from the
Monte Carlo loop; reconstruction is not repeated on every Monte Carlo input.
The relocation constructor and hashing path share caller-storage preparation,
while the textual reconstruction checks that preparation independently.

At every COUNT, inner iterations 3 through 7 and 1002, and every message
with empty updates, are directly compared with the existing single-update
public API helper. The union is 2,150 comparisons per collection. This
supplements all 100 official checkpoints and 500 published intermediate
message/digest pairs per collection. Intermediate-file final MD values are
also checked against response MD values. Failures in message hashing or
comparison report collection/source, COUNT, i, schedule slot and the complete
fragment-length list; underlying assertions report actual and expected values.

### Counts, reproduction and limitations

A complete new-target run executes 200,000 recurrence hashes (100,000 per
collection), plus 4,300 single-update differential hashes and 26 empty-free
fragmented replays: 204,326 hashes in total. Per collection, recurrence updates
are 6,420,314, including 4,689 empty calls. Supplemental checks add 12,134
updates (2,150 single calls and 9,984 replay calls). Across both collections,
there are **12,864,896 update calls**, of which 12,840,628 belong to the
recurrence. Reconstruction checks do not hash or call update.

An internal boundary means an endpoint between consecutive nonempty
fragments, excluding empty calls and the final endpoint. Per collection,
5,568,750 of 6,315,625 internal boundaries are non-byte-aligned:
**88.1741712%**. Recurrence endpoint coverage per collection is:

| Logical endpoint in bits | Occurrences |
| ---: | ---: |
| 447 | 75,000 |
| 448 | 18,750 |
| 449 | 6,250 |
| 511 | 12,500 |
| 512 | 9,375 |
| 513 | 12,500 |
| 767 | 12,500 |
| 768 (message end) | 100,000 |

Offline reproduction from the repository root:

```sh
cargo test -p kytheros_sha2 --test sha256_fragmented_monte --locked --offline -- --nocapture
cargo test --workspace --all-targets --locked --offline
```

The test reports counts and elapsed time. Observed outcomes, toolchain,
revision and full checks are in the separate
[fragmented execution record](sha256-validation-runs/2026-11-14-fragmented-monte.md).
No allocation or diagnostic formatting occurs in successful hash loops;
patterns and parsed fixtures allocate once before those loops.

The finite schedule does not exhaust all partitions, all bitstrings or all
source placements. Direct differential checks use the same cryptographic
implementation and are fault-isolation evidence, not an independent hash
oracle. Published checkpoints and intermediate values provide separate
expected values. Functional results do not establish timing, side-channel,
zeroization, fault resistance, formal CAVP/ACVTS or CMVP validation, or
embedded-hardware execution. Existing archive-authenticity limitations apply.
Human review must assess the schedule, reconstruction oracle, shared-code
extraction and evidence sufficiency; AI-generated work remains unapproved.


# Repository reorganization (November 16, 2026).

The SHA-256 source and verification directories were reorganized after the
original validation-evidence runs. Historical execution records retain their
original file paths, commands, and source checksums. The reproduction
instructions in this methodology document describe the current repository
layout. Reproducing an earlier recorded run exactly requires checking out its
recorded source revision.
