# Kytheros fragmented-update Monte Carlo execution: 2026-11-14

**Status:** Observed development evidence; human review pending.

## Tested revision and environment

- Branch: `test/sha256-fragmented-monte-carlo`.
- HEAD/start revision: `6c50b7c9e211cc3d37d0a91dadfeaf6488d1c038`.
- Initial working tree: clean. Tested state: HEAD plus the uncommitted
  test/evidence files identified below. No production source changes.
- rustc 1.98.1 (48a229cea 2026-09-01), LLVM 22.1.8.
- Cargo 1.98.1 (797e8a9bc 2026-08-05).
- Host: x86_64-unknown-linux-gnu, Linux 7.0.0-1014-gcp, x86_64.
- Default features; Cargo debug/test profiles; existing Cargo.lock.
- Bare-metal compilation only: thumbv6m-none-eabi. No hardware execution.

AGENTS.md, the documentation standard, architecture, project scope, MCL FSM,
module interfaces, existing evidence and public bit-handling implementation
were consulted. No specification conflict was identified. Requirements,
normative mappings, architectural decisions and production code are unchanged.
The [methodology](../sha256-validation-evidence.md#kytheros-fragmented-update-monte-carlo-stress-testing)
defines the schedule, sources and limits. The SHAVS §6.4 recurrence and NIST
expected values are unchanged; Kytheros designed this fragmentation procedure.
NIST does not prescribe it. No commit or push occurred.

## Changes and preservation

- Added `tests/sha256_fragmented_monte.rs`: two complete sequences plus
  finite-schedule reconstruction tests.
- Added `tests/support/monte.rs`: extracted existing parser, loader, hashing,
  recurrence and comparison helpers; imported by both integration targets.
- Changed `tests/nist_sha256_monte.rs`: import the extracted helpers. Existing
  tests and verification behavior remain unchanged.
- Extended the assurance methodology and added this separate execution record.

A mechanical comparison with HEAD confirmed shared code equality except
item/field visibility, module commentary and a trailing blank line.
Git diff confirms no production modifications. Ordinary-message tests,
fixtures, manifest, verification scripts and historical records are unchanged.
No dependency or CI workflow changes were needed. Cargo automatically includes
the new hosted integration target; no firmware includes its runtime fixtures.

## Actual results and counts

Both collections PASS. Counts below concern the new target only:

| Quantity | Per collection | Both collections |
| --- | ---: | ---: |
| Fragmented recurrence hashes | 100,000 | 200,000 |
| Official final checkpoint comparisons | 100 | 200 |
| Official intermediate message comparisons | 500 | 1,000 |
| Official intermediate digest comparisons | 500 | 1,000 |
| Intermediate-file final MD/response MD comparisons | 100 | 200 |
| Recurrence update calls | 6,420,314 | 12,840,628 |
| Empty update calls (included above) | 4,689 | 9,378 |
| Internal nonempty fragment boundaries | 6,315,625 | 12,631,250 |
| Non-byte-aligned internal boundaries | 5,568,750 | 11,137,500 |
| Relocated recurrence messages | 100 | 200 |
| Single-update differential hashes/comparisons | 2,150 | 4,300 |
| Empty-free fragmented replay hashes/comparisons | 13 | 26 |
| Supplemental update calls | 12,134 | 24,268 |
| Total hashes | 102,163 | 204,326 |
| Total update calls | 6,432,448 | 12,864,896 |

The boundary fraction is **88.1741712%**, excluding empty calls, final
endpoints and supplemental hashes. Each of the 32 slots occurs 3,125 times
per collection, including 3,125 messages with 768 single-bit updates and
3,125 with 256 three-bit updates. All messages use at least two nonempty
updates. The schedule includes every required asymmetric pair and irregular
length, plus mixed small/large fragments.

The 100 relocated messages and 13 relocated replays per collection use
source offsets 1 through 8 as fragment ordinal varies. Separately, all 32
partition slots were reconstructed on all 768 one-hot inputs: 24,576 cases,
with no hashing/update calls. These checks verify consumption, order, no
skipped/repeated positions, exact concatenation, final offset 768 and MSb-first
relocation using independent binary-text decoding. Reconstruction is not
repeated on every Monte Carlo input.

| Recurrence endpoint in bits | Occurrences per collection |
| ---: | ---: |
| 447 | 75,000 |
| 448 | 18,750 |
| 449 | 6,250 |
| 511 | 12,500 |
| 512 | 9,375 |
| 513 | 12,500 |
| 767 | 12,500 |
| 768 (final endpoint) | 100,000 |

All direct single-update comparisons and empty-free replay comparisons
passed. Every selected empty-update message was directly compared with a
single-update digest; 13 per collection also repeated the same fragmented
hash without empties. No official value or production code was changed.

## Commands actually executed

All final commands below exited 0 from the repository root:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --all-targets --locked --offline -- --nocapture
cargo test --doc --workspace --locked --offline
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --workspace --no-deps --locked --offline
cargo check --workspace --locked --offline --target thumbv6m-none-eabi
python3 crates/cryptographic_primitives/kytheros_sha2/tests/fixtures/nist/verify.py
python3 crates/cryptographic_primitives/kytheros_sha2/tests/fixtures/nist/verify_monte.py
```

The new target also passed separately with:

```sh
cargo test -p kytheros_sha2 --test sha256_fragmented_monte --locked --offline -- --nocapture
```

The exact CI find/awk 80-character Rust-source check passed. Git diff
whitespace checking passed. Added relative documentation file links resolve
locally; the section anchor was manually checked. Rustdoc passed with broken
intra-doc links denied. No required checks were omitted. All tests run offline;
none are ignored and missing fixtures fail.

| Test group | Passed | Failed/ignored | Final workspace test duration |
| --- | ---: | --- | --- |
| Existing unit tests | 30 | 0/0 | 0.46 s |
| Existing ordinary-message tests | 6 | 0/0 | 2.11 s |
| Existing NIST Monte Carlo tests | 5 | 0/0 | 3.82 s |
| New fragmented tests | 3 | 0/0 | 6.65 s |
| Documentation tests | 8 | 0/0 | 0.53 s command wall time |

The complete workspace test command took 14.11 s including compilation.
Within concurrent fragmented tests, bit collection elapsed time was 6.214 s
and byte collection elapsed time was 6.650 s. These are local observations,
not performance guarantees. No slow-test feature or coverage reduction was
needed. Both original Python hashlib sequences and intermediate checks
passed; Python integrity checks passed both archives and all ten fixtures.
Expected caught panics from existing negative tests appear under nocapture.

An initial Clippy check identified a needless indexed loop in the new
reconstruction test. It was changed to an enumerated iterator before final
checks. Binary byte text was cached to reduce reconstruction overhead;
coverage was preserved.

## Limitations and human review

No production defect, unexpected digest behavior, failed official vector or
unresolved ambiguity was detected. The finite schedule does not exhaust all
partitions, bitstrings or source placements. Direct differential checks use
the same SHA-256 implementation and are not an independent hash oracle.
Not every inner digest receives a direct single-update comparison; all
published checkpoints and intermediate values do. Archive authenticity
remains independently unverified.

These results establish functional observations, not formal CAVP/ACVTS or
CMVP validation, production approval, timing/side-channel properties,
zeroization assurance or hardware execution. Human review must assess the
shared-code extraction, schedule, reconstruction independence, alignment
coverage and evidence sufficiency. AI-generated work remains unapproved
until accepted through project review.

## Tested source integrity identifiers

Python hashlib SHA-256 values identify uncommitted test/evidence inputs.
The base revision identifies unchanged production source and fixtures.
These identifiers do not independently authenticate publication.

| File | SHA-256 |
| --- | --- |
| `crates/cryptographic_primitives/kytheros_sha2/tests/sha256_fragmented_monte.rs` | `afca043123cce2da5078930a8ee5041cc38f1b93ee443311dbc9dca5f8a7fb3e` |
| `crates/cryptographic_primitives/kytheros_sha2/tests/support/monte.rs` | `80b5d40ff7acb17afd451c71e4646e89f68d78fd6d54a9dac37b3fe602e52899` |
| `crates/cryptographic_primitives/kytheros_sha2/tests/nist_sha256_monte.rs` | `ac827b85433ce6a174fbf8d9cf3a94cc576ec6fe962fe1ae5193aabf8d8675fa` |
| `docs/assurance/sha256-validation-evidence.md` | `8ea9eff94a814c68437aa6b68c06ef23a99bedfb6ce9a109ca29e7d06b86080e` |
| `Cargo.lock` | `d27572c9ecd35e2a77b0c1ea280b56a00c6121bc8629796e5da994d8f0c235e4` |
| `rust-toolchain.toml` | `1c5430ce1b60fb91c62d1e1e6ca5e6e81934e7fc181ef737cbfd8d679f47af89` |
