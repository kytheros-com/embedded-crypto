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

## 3. Logical boundary and PIL design gates

- [ ] Define fixed mailbox layout and protocol records for five separate
  logical-interface payloads; retain the MCL/external permission matrix.
- [ ] Specify each mailbox's independent atomic synchronization object,
  permissions, producer/consumer transitions and acquire/release ordering.
- [ ] Specify bounded acquisition and internal-only validation of untrusted
  input, including adversarial concurrent writes, torn snapshots, malformed
  control records and refusal to re-read accepted external request fields.
- [x] Agree that success, authentication, context commit and
  authorization of the **entire result** precede the first outward
  byte-copy. Semaphore publication is cooperative synchronization only;
  exact encoding and target inhibition evidence remain pending.
- [ ] Verify that the resulting fixed-permission mailboxes do not disclose
  prohibited SSPs or violate CMVP data/control-output inhibition in error,
  pre-operational testing, zeroization, manual entry or loading conditions.
  Protocol-level nonpublication alone is not proof of output inhibition.
- [ ] Design mailbox transaction invalidation/recovery following errors,
  resets and interrupts without trusting stale retained memory.
- [ ] Define the control/status/cryptographic-output message classification,
  and deferred mailbox sizes, formats and checksum/EDC threat model.
- [ ] Document the Level 1 backup-operational limitation: an adversarial
  host choosing or learning the password-derived wrapping key can recover
  encrypted exported SSPs. An organizational root-authority/trusted-recipient
  recovery design is outside the initial scope.
- [ ] Validate the representative STM32U5 PIL memory isolation, DMA/bus
  protections, reset paths and relevant errata after hardware arrives.
  This does not block continued architecture design.

## 4. Documentation and assurance reconciliation

- [ ] Review and approve `docs/assurance/configuration-management.md`,
  separating permanent process rules from this task list.
- [ ] Reconcile the legacy `SR-...` identifiers, revision/retrieval data and
  source-maintenance rules from the deleted Markdown standards register with
  the workbook's `Source Register`; preserve any unique information before
  final acceptance of its deletion. Git history retains the former register.
- [ ] Reconcile the requirements workbook's source statuses and algorithm
  coverage against the architectural layer assignments.
- [ ] Record today's five-mailbox design, atomic synchronization contracts
  and PIL isolation requirements in the workbook's existing requirements,
  decisions, questions and evidence records. The binary workbook is
  intentionally not modified by the documentation-only GitHub update.
- [ ] Review the provisional `requirement-ownership.csv` mapping and build
  requirement-to-design-to-test links without changing stable workbook IDs.
- [ ] Determine whether `project-scope.md`, `architecture.md`, and the
  reference integration's eventual boundary remain fully consistent.

## 5. MCL finite-state machine — conceptual baseline agreed

- [x] Preserve normative FSM research in
  `docs/assurance/fsm-requirements-notes.md`.
- [x] Record module, operation, context and key lifecycles, operation
  acceptance, working-copy commit, exceptional outcomes, one-way
  `RETIRING`, predefined bounded composition groups and the initial
  single-executor/future-concurrency distinction in
  `docs/design/mcl-fsm.md`.
- [x] Record both IV strategies, irreversible spent allocations and
  `DECRYPT_ONLY`, RAM-bounded whole-message GCM decryption,
  complete-result authorization before outward byte copy, and
  semaphore-only synchronization.
- [x] Record scope-specific fault zeroization, distinction between
  `ERROR` and module-wide purge, unconditional volatile context reset
  cleanup and durable per-region interrupted-erasure markers.
- [ ] Complete exhaustive state/event/guard/action/output table with
  catch-all transitions and specific services, roles, authentication,
  role clearing, conditional/on-demand self-tests and status indicators.
- [ ] Select mailbox transfer formats, atomic ordering and actual
  publication mechanics; test data/control-output inhibition, stale
  mailbox bytes and interrupted outward copying.
- [ ] Verify target PIL reset, processor fault, DMA/stack access,
  primitive-writable inventory, source-level and compiled zeroization.
- [ ] Select bounded decryption `L_max` after MCU RAM budgeting;
  implement persistent key/IV accounting, anti-rollback and shared-key
  aggregate RBG invocation limit policy.
- [ ] Define marker encoding and durable update sequence for actual
  storage technology; test resets/torn writes during erasure.
- [ ] **Reconcile these decisions with the existing binary workbook**
  requirements, design decisions, service contracts, SSP plan, open
  questions and evidence; preserve identifiers. **Workbook not
  modified in this documentation commit.**
- [ ] Link final transition tests and design to existing D-007–D-009,
  Q-001, Q-008 and applicable workbook evidence IDs.
