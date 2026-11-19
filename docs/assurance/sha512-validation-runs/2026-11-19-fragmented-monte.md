# SHA-512 fragmented Monte Carlo execution, 2026-11-19

**Status:** Observed internal development evidence. Human review pending.

The [main execution record](2026-11-19.md) identifies the environment,
source checksums and shared verification commands. The tested revision
was `sha512` at `10a33cb662e2eefd3a44357a519ebd68757c91f0` plus the
uncommitted SHA-512 test and fixture inputs. The
[methodology](../sha512-validation-evidence.md#kytheros-fragmented-monte-carlo)
defines the Kytheros partition schedule and its relationship to the
published recurrence and expected values.

## Observed coverage

Both collections passed. Each of 32 schedule slots ran 3,125 times per
collection, and every recurrence hash used at least two nonempty
updates. The independent one-hot reconstruction completed all 49,152
cases in one test, across all 1,536 positions and 32 slots.

| Quantity | Per collection | Both |
| --- | ---: | ---: |
| Fragmented recurrence hashes | 100,000 | 200,000 |
| Published response checkpoints | 100 | 200 |
| Published intermediate M values | 500 | 1,000 |
| Published intermediate MDi values | 500 | 1,000 |
| Detailed final MD / response MD agreements | 100 | 200 |
| Recurrence update calls | 12,057,814 | 24,115,628 |
| Recurrence empty calls | 4,689 | 9,378 |
| Internal nonempty fragment boundaries | 11,953,125 | 23,906,250 |
| Non-byte-aligned internal boundaries | 10,484,375 | 20,968,750 |
| Relocated recurrence messages | 100 | 200 |
| Single-update consistency comparisons | 2,150 | 4,300 |
| Empty-free fragmented replay comparisons | 13 | 26 |
| Supplemental updates | 22,118 | 44,236 |
| All update calls | 12,079,932 | 24,159,864 |
| All hashes | 102,163 | 204,326 |

Internal boundary counts exclude empty calls, final endpoints and
supplemental comparisons. The 100 relocated recurrence messages
exercised all eight source alignments. The 13 empty-free supplemental
replays per collection also used relocation. The independently
enumerated schedule agreed with the observed coverage counts.

| Endpoint in bits | Occurrences per collection |
| ---: | ---: |
| 895 | 15,625 |
| 896 | 6,250 |
| 897 | 15,625 |
| 1023 | 9,375 |
| 1024 | 9,375 |
| 1025 | 9,375 |
| 1281 | 9,375 |
| 1534 | 6,250 |
| 1535 | 6,250 |
| 1536 | 100,000 |

Relocated nonempty-fragment alignment counts for residues 0 through 7
were `[5375, 5425, 5425, 5400, 5400, 5400, 5400, 5400]` per collection.
The same-implementation supplemental comparisons establish consistency
for selected inputs. Published comparisons and the separate Python
history-list replay supply independent digest checks.

## Commands, cost and limits

```sh
cargo test -p kytheros_sha2 --test sha512_fragmented_monte --locked
cargo test -p kytheros_sha2 --test sha512_fragmented_monte --locked -- --nocapture
cargo test -p kytheros_sha2 --test sha512_fragmented_monte --locked -- partition_reconstruction_and_relocation --exact --nocapture
```

All exited 0. The first two commands each ran all three tests. The
third measured just the reconstruction test for timing. The workspace
command also ran all three. The individual full target took 14.27 s
and the logged rerun took 14.74 s of test execution. During the logged
run, the bit and byte collection tests took 14.74 s and 12.93 s
respectively and ran concurrently with reconstruction. Reconstruction
took 7.36 s when measured alone. The principal workloads are the full
Monte Carlo recurrence with over 24 million update calls and the
49,152 independently reconstructed cases. These are host timings
under the development test profile. The final checks retained all
vectors and reconstruction cases.

An earlier exploratory full run passed before counters were tightened
to the independently enumerated totals. No official expected value
changed. Human review should assess partition coverage, relocation,
reconstruction independence and parser interpretation. The observed
coverage is limited to the finite deterministic schedule and host
configuration.
