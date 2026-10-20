# Configuration Management and Assurance Change Control

**Status:** Draft for design review  
**Date:** 2026-10-20  
**Scope:** `kytheros-com/embedded-crypto`

## 1. Purpose and authority

This document defines the proposed process for controlling the project's
engineering and assurance records. It complements
[project-scope.md](project-scope.md) and
[architecture.md](architecture.md) without duplicating the normative
requirements or source inventory held in
[the CMVP requirements workbook](CMVP_Level_1_Software_Requirements_Matrix.xlsx).

The workbook is the authoritative planning register for source identities,
requirements, design decisions, applicability, open questions, planned services
and evidence status. The approved architecture controls component contracts.
An approved algorithm design controls the corresponding primitive's detailed
behavior. Inconsistencies are recorded and resolved; they are not settled by
silently copying or overriding entries.

## 2. Branch, review and change acceptance

- Propose changes on a branch; review them through a pull request before
  merging into protected `main`.
- State the reason for each substantive change, the affected layers, and
  its effect on security contracts and assurance evidence.
- Obtain review appropriate to the risk and expertise required. Any change
  that alters normative applicability, the future validation boundary, or
  a claimed validated configuration requires relevant independent or CSTL
  consultation where applicable.
- Do not treat a successful build, passing CI, or a planning document as
  evidence that a design decision was implemented, reviewed or validated.
- Preserve a reproducible identification of the source revision, toolchain,
  build flags, target, dependencies, implementation variants and evidence
  associated with any reviewed executable artifact.

## 3. Stable identifiers and traceability

- Requirement, source, design-decision, question, service, SSP and evidence
  identifiers are stable. Never reuse an existing identifier for a different
  subject.
- Record supersession, changed applicability and source revisions
  explicitly; do not rewrite history by silently repurposing an ID.
- Reconcile the accepted mailbox architecture and platform-independent PIL protection properties with the workbook's current design decisions, questions and evidence plan before claiming complete traceability.
- Link each applicable requirement to design owners and component-level
  implementation and test evidence. For obligations shared across layers,
  preserve the parent requirement ID and distinguish the separate evidence
  claims.
- Keep source identities and version metadata in the workbook rather than
  creating a second authoritative Markdown standards register.

## 4. Security-impact review

For each security-relevant change, assess as applicable:

- module, primitive and PIL interfaces, five mailboxes, their atomic synchronization objects, and authorization boundaries;
- inbound snapshot coherence and safe outbound publication even when external clients ignore synchronization rules;
- SSP lifetimes, permitted read/write regions, aliases and memory ownership;
- cleanup/zeroization on completion, internal fault, reset or interruption;
- fault/exception-vector control, trusted recovery, retained FSM state, and mailbox transaction invalidation;
- randomness/entropy requirements and DRBG construction or configuration;
- supported algorithm parameters, implementations and compiled variants;
- side-channel and fault assumptions, compiler behavior and stack/RAM usage;
- module roles, services, error/output inhibition and self-test sequencing;
- target and operational-environment differences, documentation and guidance;
- test coverage, CAVP/CMVP impact and previously gathered assurance evidence.

Material effects require an updated requirements/evidence cross-reference
and appropriate tests before declaring the change complete.

## 5. Source and evidence maintenance

- Record issuer, title, revision, publication/source identifier and stable
  clause or section for each applicable source in the workbook.
- Recheck mutable CMVP and algorithm-validation guidance at defined assurance
  release gates and before any submission; record the retrieval date.
- Retain an exact acquired artifact and cryptographic digest when mutable
  guidance is relied upon as evidence, subject to applicable licensing.
- Do not publish licensed source texts without redistribution permission.
- Assess source revisions for design, testing, validation and evidence impact.
- Keep unresolved source applicability questions explicitly open.
- Preserve the legacy `SR-...` source identifiers and source-maintenance
  information from the removed `standards-register.md` until their mapping
  to the workbook has been reviewed. Git history retains the original file.

## 6. Completion and approval

Evidence status is set on the strength of identified work products and
review, not because a task was scheduled or a document was created. A
configuration-management decision itself becomes binding only after its
review and acceptance through the normal change-control process. The list
of outstanding work belongs in `docs/todo.md`; the workbook remains
authoritative for detailed requirement and design-decision status.
