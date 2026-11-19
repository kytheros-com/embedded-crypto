# SHA-512 published-vector verification methodology

**Status:** Internal development methodology. Human review pending.

## Scope and references

The hosted harness exercises the public `BitSpan` and `Sha512Context`
interfaces of `kytheros_sha2`. It covers full-length SHA-512, including
arbitrary-bit input, streaming, padding, and digest serialization.
FIPS 180-4 (August 2015) defines bit and word representation in §§3.1–3.2,
the SHA-512 functions and constants in §§4.1.3 and 4.2.3, padding and
block parsing in §§5.1.2 and 5.2.2, the initial hash value in §5.3.5,
and the hash computation in §6.4. [FIPS 180-4][fips]

SHAVS, updated May 21, 2014, defines the ordinary and Monte Carlo
procedures in §§6.2–6.4. The response grammar is based on the retained
archive members. [SHAVS][shavs]

The repository's [documentation standard](../documentation-standard.md),
[architecture](architecture.md), and [project scope](project-scope.md)
govern development evidence. The architecture's CMVP workbook remains
the controlling assurance planning baseline. The harness runs outside
the primitive and exercises neither MCL service policy nor PIL mechanisms.

## Sources and byte integrity

The test inputs are the two NIST archives retained in
`crates/cryptographic_primitives/kytheros_sha2/tests/NIST_KAT_Files/`.
The earlier SHA-256 evidence records that the user supplied these archives
in November 2026. Their original download date and independent
authentication remain unknown. This SHA-512 verification uses those
retained bytes. [NIST CAVP Secure Hashing][cavp]
[Original bit archive][bit-zip] · [Original byte archive][byte-zip]

| Archive | SHA-256 |
| --- | --- |
| `shabittestvectors.zip` | `cd7b9f11680c6e0ccdbe13b28403f2017b5ff48789152162461e0a24fb4c5d45` |
| `shabytetestvectors.zip` | `929ef80b7b3418aca026643f6f248815913b60e01741a44bba9e118067f4c9b8` |

For each archive, the selection is
`Readme.txt`, `SHA512ShortMsg.rsp`, `SHA512LongMsg.rsp`,
`SHA512Monte.rsp` and `SHA512Monte.txt`. Each retains its archive-internal
directory and original bytes under `tests/sha512_tests/fixtures/nist/`.
The similarly named SHA-512/224 and SHA-512/256 files are outside this
algorithm scope. The ten selected members use CRLF. Directory-local Git
attributes preserve those bytes across checkouts.

The [provenance manifest][manifest] identifies archive URLs, checksums,
member paths, SHA-256 fingerprints, newline counts and published case
counts. The Python `verify.py` independently calculates SHA-256 with
`hashlib`, checks both archive checksums, verifies manifest completeness,
and checks each retained file against its original ZIP member byte for
byte, including line endings. The Rust loaders additionally calculate
SHA-512 fingerprints over the fixture files with the public API and
compare them to values calculated with `hashlib.sha512` from the
extracted bytes. These fingerprints are regression identifiers and the
manual archive check supplies independent integrity evidence. Neither
byte equality nor matching checksums authenticates the original source.

The two archived READMEs have identical bytes. They identify the CAVS
response files and describe the first five tab-indented intermediate
`i`, `M`, `MDi` groups per COUNT and the unindented final `MD`.
Partial-byte placement and the zero-length placeholder are established
separately from the response data and the FIPS bit order.

## Ordinary messages and incremental replay

SHAVS §4.2 defines `Len` in bits and `L` in digest bytes. The parser
requires one `[L = 64]` section and ordered `Len`, `Msg`, and `MD`
fields, with blank separators and header comments as in the original
files. It checks decimal lengths, hexadecimal encoding, exact storage
sizes and counts. Unexpected fields, sections, extra cases, truncated
records and absent files fail. Synthetic negative inputs exercise the
parser independently of the vector digests.

| Collection | Member | Cases | Length in bits |
| --- | --- | ---: | --- |
| Bit | `SHA512ShortMsg.rsp` | 1,025 | 0 through 1,024 |
| Bit | `SHA512LongMsg.rsp` | 1,024 | 1,123 through 102,400 by 99 |
| Byte | `SHA512ShortMsg.rsp` | 129 | 0 through 1,024 by 8 |
| Byte | `SHA512LongMsg.rsp` | 128 | 1,816 through 102,400 by 792 |

For positive lengths, `Msg` contains exactly `ceil(Len/8)` bytes. The
high bits of the final byte hold meaningful bits, and the archived
unused low bits are zero. The parser checks that convention. The
`Msg = 00` placeholder at `Len = 0` contributes no bits. Each case
uses a fresh context and caller-owned 16-word schedule workspace and
compares all 64 serialized digest bytes in H_0 through H_7 order.

The ordinary target exercises all 2,306 cases. It also replays all
2,049 bit-oriented cases using a Kytheros-selected repeating update
sequence of 1, 7, 8, 55, 895, 896, 1023, 1024 and 1025 bits. The harness
clips only the last fragment at the endpoint and submits an empty call
for an empty message. The expected digests remain the published values.
This sequence exercises partial bytes, changing alignments and multiple
blocks. The finite set does not cover all possible update partitions.

## SHAVS Monte Carlo

The two Monte Carlo collections have different seeds and checkpoints.
As specified in SHAVS §6, both recurrences use byte-oriented messages.
For each COUNT 0–99, initialize three digest words of history with the
current 64-byte seed. For each `i` from 3 through 1002, concatenate
`MD[i-3] || MD[i-2] || MD[i-1]`, preserving the published byte order,
and hash the resulting 192-byte (1,536-bit) message. Compare `MD[1002]`
to the response digest, then use it as the next seed and reset all three
initial slots. This is the procedure in SHAVS §6.4, Figure 1. [SHAVS][shavs]

Each collection specifies 100 checkpoints, with 1,000 hashes per
checkpoint. The detailed `SHA512Monte.txt` supplies `M` and `MDi`
for `i = 3..7` at every COUNT, for 500 pairs per collection. The
harness compares both values at each published index and verifies that
the detailed final `MD` agrees with the response `MD`. There are no
published intermediate pairs in these archives for `i = 8..1002`.
The final response covers the digest at `i = 1002`.

The ordered parser requires the digest section, a 64-byte seed, exactly
100 ordered COUNT records, 64-byte MD fields and, in detailed files,
all five ordered tab-indented groups with 192-byte M fields and
64-byte MDi fields. The two Rust Monte Carlo targets share this
SHA-512-specific parser and recurrence. The single-update target uses
fixed-size message and digest arrays and the public hashing API.
Synthetic tests exercise byte order, digest shifting, seed reset,
parser errors and checkpoint mismatch detection.

The manual Python `verify_monte.py` performs the same published
comparisons with `hashlib.sha512` and a full digest history for the
current COUNT. It neither reads nor calls Rust production code.
The archive values are the expected checkpoints and intermediate
pairs. Python-computed digests are separate reference calculations.

## Kytheros fragmented Monte Carlo

This extension supplies each message in multiple nonempty calls and
retains the original seeds, recurrence, and expectations. For
`n = COUNT * 1000 + i - 3`, select slot `n % 32` independently in
each collection.

| Slots | Nonempty fragment lengths in bits |
| --- | --- |
| 0 | 1,536 one-bit fragments |
| 1 | 512 three-bit fragments |
| 2 | Irregular sequence from rotation zero |
| 3 | 13, 1011, 1, 511 |
| 4–13 | Two fragments. First endpoint is 1, 895, 896, 897, 1023, 1024, 1025, 1281, 1534 or 1535, in that order |
| 14–31 | Irregular sequence from rotation `(slot - 14) % 9` |

The irregular sequence repeats `[1, 3, 5, 7, 9, 13, 17, 31, 63]` from
the indicated rotation and clips its last length at the final endpoint.
Every slot occurs 3,125 times per collection. Endpoints must advance
strictly and finish at 1,536. The tests account for every nonempty
source alignment, invalid endpoints cause failure, and internal
boundaries exclude the final endpoint and empty calls.

When `n % 64 == 0`, place empty updates before the first fragment,
after it, and at the message endpoint. When `n % 1000 == 0`, relocate
each fragment to bit offset `1 + ordinal % 8` in a separate 193-byte
buffer initialized with a nonzero sentinel. The offset cycle covers all
eight alignments. At published intermediate indices, at final
checkpoints, and for messages with empty updates, compare the
fragmented digest with a single-update digest. When `n % 8000 == 0`,
repeat the same fragmentation without empty updates. These
same-implementation comparisons check consistency. Independent digest
evidence comes from the published expectations and Python replay.

The reconstruction test traverses all 32 slots for every one of the
1,536 one-hot input positions. It verifies bounds and visits each input
position once in order. It independently decodes relocated bytes as
binary text and compares the reconstructed message to the original
MSb-first bitstring. This gives 49,152 harness checks without hashing.
It exhausts the finite schedule, with coverage limited to the selected
partitions and basis inputs for reconstruction. The recurrence tallies
hashes, expected comparisons, update calls, empty calls, internal
boundaries, unaligned boundaries, source alignments and relocated
messages. Supplemental calls are counted separately.

## Reproduction and evidence limits

From the repository root:

```sh
fixture_root=crates/cryptographic_primitives/kytheros_sha2/tests/sha512_tests/fixtures/nist
python3 "$fixture_root/verify.py"
python3 "$fixture_root/verify_monte.py"
cargo test -p kytheros_sha2 --test nist_sha512 --locked -- --nocapture
cargo test -p kytheros_sha2 --test nist_sha512_monte --locked
cargo test -p kytheros_sha2 --test sha512_fragmented_monte --locked -- --nocapture
cargo test --workspace --all-targets --locked
```

Cargo declares all three hosted tests explicitly. The existing CI command
includes them. Tests require neither network access nor Python. Fixture
I/O belongs to the hosted harness. Existing unit-test fixtures, including
locally derived arbitrary-bit digests, are separate development evidence
and retain their own provenance.

See [execution records](sha512-validation-runs/) for the tested revision,
counts, timings, and observed outcomes. This methodology supports
internal functional evidence. Formal CAVP or ACVTS work, FIPS 140-3 and
CMVP validation, side-channel analysis, zeroization assurance and
production approval require separate evidence. Cross-compilation checks
compilation only. Human review must assess provenance, grammar,
recurrence, partition reconstruction and evidence sufficiency.

[fips]: https://doi.org/10.6028/NIST.FIPS.180-4
[shavs]: https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/SHAVS.pdf
[cavp]: https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/secure-hashing
[bit-zip]: https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabittestvectors.zip
[byte-zip]: https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabytetestvectors.zip
[manifest]: ../../crates/cryptographic_primitives/kytheros_sha2/tests/sha512_tests/fixtures/nist/provenance.json
