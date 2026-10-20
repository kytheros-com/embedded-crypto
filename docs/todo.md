## 1. Initial implementation milestone

**First implementation focus: SHA-2** in `crates/kytheros_sha2`, using the current workspace as the starting point.

The SHA-2 milestone shall establish, to the extent applicable:

- A frozen list of implemented variants and exposed APIs, with their governing specification identified in the workbook (see Q-007).
- Explicit incremental/finalization and invalid-input behavior, with bounded state and no mandatory `std` or `alloc` use.
- Documented input/output, alignment, workspace, overlap, copy and cleanup contracts.
- Known-answer, empty-input, padding-boundary, incremental, maximal-length/overflow-handling (as feasible), and negative tests relevant to the chosen APIs.
- Repeatable compiler/configuration records, compilation checks for selected embedded configurations, and initial resource measurements once a representative target exists.
- Honest claims limited to what the implemented code, tests, and reviews actually establish.

This milestone does **not** require implementation of entropy hardware, Hash_DRBG, key backup, a full module finite-state machine, or a completed CMVP submission package before its algorithmic work can be evaluated. Subsequent module integration must satisfy its own requirements.

## 2. Unresolved design gates

The following decisions remain **open**, notwithstanding the agreed three-layer division:

| Gate | Issue | Workbook references |
| --- | --- | --- |
| Actual cryptographic/privilege boundaries and OE classification | Hardware/firmware embodiment, included trusted components, physical applicability | Q-001; D-001, D-007 |
| RBG approval, entropy-source match and recovery | Construction/credit, purchased device and conditions, demand bounds and fault behavior | Q-002, Q-003, Q-006, Q-008 |
| External ML-KEM shared-secret contract | Handle/consumer interface versus approved protected output, including validation implications | Q-005; D-010 |
| Stored-key backup | Protection format, SP 800-132 selection, password/input contract and strength justification | Q-004, Q-009, Q-010; D-011/D-012 |
| Algorithm variants and test coverage | Exposed hash APIs/acceleration, PQC key checks, seed restoration | Q-007, Q-011 |

The workbook governs their current decision statuses. No implementation may turn an open gate into an unstated assumption.

## 3. Configuration management and review

ChatGPT has made the following proposals which seem good to me. However, I'm not sure where they belong. They don't seem to fit in the architecture.md since they are process guidance rather than design architecture. Figure out the correct place to document this.

- Architectural changes shall be proposed and reviewed on a branch before merge to protected `main`.
- Requirement, decision, source, question and evidence identifiers shall remain stable; add cross-references rather than duplicate authoritative tables.
- Each change shall identify affected layers, public/internal contracts, threat and leakage assumptions, target configurations, tests, and evidence.
- If a decision changes normative applicability or the future validation boundary, update the corresponding workbook entries and obtain independent/CSTL assessment when applicable.
- Changes to supported parameters, memory ownership, secret output, zeroization, randomness, hardware variants, or module services require explicit security-impact review.
- Implementation and evidence statuses must never be inferred merely from the existence of a design document, CI success, or a completed planning worksheet.

## 4. Design-decision and requirement cross-reference

Need to cross reference assurance/architecture.md with the Workbook for consistency and requirements tracing.