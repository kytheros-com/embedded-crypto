# Project Assurance Scope

## Purpose

The `embedded-crypto` project develops reusable cryptographic algorithm
implementations in Rust, together with technical evidence intended to support
their review, integration, testing, and potential use in cryptographic modules
submitted for third-party validation.

The project may also develop a target-specific Platform Integration Layer
(PIL) and Module Control Layer (MCL) for a reference cryptographic module.
These layers are described in [architecture.md](architecture.md), and the
reference module's fixed logical service interfaces are specified in
[module-interfaces.md](../design/module-interfaces.md). Their
development does not make the reusable primitives or reference integration
independently validated, and their inclusion in a future validated module
depends on that module's identified cryptographic boundary.

The project is designed to make the algorithm implementations and their
supporting evidence understandable and usable by:

* cryptographic module developers;
* security reviewers;
* testing laboratories;
* system integrators; and
* other technically qualified users.

Planned algorithms, parameter sets and operations are recorded in the
`Source Register` and `Algorithm Profile` worksheets of
[the CMVP requirements workbook](CMVP_Level_1_Software_Requirements_Matrix.xlsx).
These worksheets, rather than a duplicate Markdown standards register,
provide the authoritative planning record. A planned capability is not
represented as implemented or validated without separate evidence.

## Project Status

This repository is an open-source development project. Neither this repository
nor any crate, source file, test result, build artifact, or release is claimed
to be:

* validated under FIPS 140-3;
* validated by the Cryptographic Module Validation Program (CMVP);
* validated by the Cryptographic Algorithm Validation Program (CAVP);
* approved for protecting United States Government information;
* compliant with the Commercial National Security Algorithm Suite 2.0 merely
  because it implements an algorithm named by that suite; or
* suitable for production use without an independent assessment of the
  implementation and its integration.

Passing project tests, including known-answer tests, demonstrates only the
specific properties exercised by those tests. It does not constitute CAVP or
CMVP validation.

## Validation Model

FIPS 140-3 validation applies to a precisely identified cryptographic module
with a defined cryptographic boundary, version, operational environment,
configuration, set of services, and implementation.

CAVP algorithm validation applies to a particular algorithm implementation in
a particular operational environment. Source code compiled into different
binary artifacts or incorporated into different module configurations may
require separate testing or validation.

This repository develops validation-supporting algorithm components and
evidence, and may additionally develop a concrete embedded reference
integration. Neither a reusable component nor an integration is a universally
validated cryptographic module.

Whoever submits an actual module for FIPS 140-3 validation, whether the
reference-module developer or a third-party integrator, remains responsible
for working with an accredited Cryptographic and Security Testing Laboratory
to establish applicable requirements and evidence for that specific module.

## Reference module logical interface boundary

The optional reference module uses five separate fixed-memory mailbox
payloads for control input, control output, status output, data input
and data output. The MCL exclusively mediates these logical service
interfaces while the platform-specific PIL enforces memory isolation,
atomic synchronization and fault/reset entry. Portable cryptographic
primitive crates accept authorized internal buffers, **never pointers
to externally accessible reference-module mailboxes**.

The logical service boundary does not itself establish the eventual
FIPS 140-3 cryptographic boundary or physical perimeter. No trusted
channel/path implementation is committed for the initial design.
External callers, including a compromised orchestrating host, are not
trusted to follow mailbox protocols. MCL service decisions use validated
internal input copies; it checks outbound information before copying it
into externally readable mailbox memory and publishes the message through
the defined atomic synchronization step. Early external reads do not
constitute valid published results, but prohibited SSP disclosure and
normative output inhibition remain forbidden. Mailbox protocols and
hardware effectiveness remain subject to further design and verification.

## Intended Assurance Artifacts

Where applicable, the project intends to provide:

* the applicable specification and revision for each implementation;
* traceability from applicable specification requirements to design, code,
  tests, and evidence;
* documented public interfaces, preconditions, postconditions, invariants,
  and error behavior;
* descriptions of internal state, data flow, message processing, padding, and
  encoding;
* known-answer, boundary, incremental, differential, property, and negative
  tests;
* test coverage for failure and error-handling paths;
* adapters and capability descriptions suitable for development with the
  Automated Cryptographic Validation Protocol;
* cryptographic algorithm self-test vectors and integration guidance;
* descriptions of timing, memory, stack, and other implementation properties;
* analysis of side-channel-relevant control flow and memory access;
* exact build inputs for identified release artifacts;
* dependency, license, and software-bill-of-materials information;
* change-impact assessments for security-relevant modifications; and
* integration guidance identifying obligations not satisfied by the algorithm
  component itself.

The presence of an intended artifact in this list does not imply that the
artifact has been completed or independently verified. Artifact status is
recorded separately in the project requirements and evidence records.

## Division of Responsibilities

The columns below distinguish **reusable algorithm components** from
**module-level integration**. A reference MCL/PIL integration developed within
this repository assumes the module-integrator responsibilities applicable to
that reference embodiment; a third-party integrator independently assumes
those responsibilities for its own implementation and security boundary.

* **Algorithm specification**

  * **Reusable components:** Identify and trace applicable requirements.
  * **Module integrator:** Confirm that selected algorithms and uses are
    permitted.

* **Algorithm implementation**

  * **Reusable components:** Provide reviewed source and identified release
    artifacts.
  * **Module integrator:** Incorporate an exact implementation into the
    module.

* **Development testing**

  * **Reusable components:** Provide documented automated tests.
  * **Module integrator:** Evaluate applicability and perform integration
    testing.

* **CAVP testing support**

  * **Reusable components:** Provide test adapters and capability information where
    practical.
  * **Module integrator:** Obtain official testing through an authorized
    laboratory.

* **Cryptographic algorithm self-tests**

  * **Reusable components:** Provide vectors, callable implementation paths, and
    integration guidance.
  * **Module integrator:** Control invocation, pass/fail determination,
    service gating, and failure response.

* **Cryptographic boundary**

  * **Reusable components:** Describe component boundaries and dependencies;
    do not depend on external-interface mailboxes.
  * **Module integrator:** Define and document the formal validated module
    boundary. For the reference implementation, the MCL mediates the five
    mailbox-based logical interfaces and the PIL enforces their access controls.

* **Approved mode**

  * **Reusable components:** Avoid making an independent approved-mode claim.
  * **Module integrator:** Define and enforce approved operation.

* **Roles and services**

  * **Reusable components:** Describe algorithm operations exposed by the
    component.
  * **Module integrator:** Define module roles, services, authorization, and
    authentication.

* **Service indication**

  * **Reusable components:** Document algorithm success and failure behavior.
  * **Module integrator:** Provide the required approved-service
    indication.

* **Module state machine**

  * **Reusable components:** Document component state transitions.
  * **Module integrator:** Define and enforce the module finite-state
    model.

* **Software or firmware integrity**

  * **Reusable components:** Support deterministic identification of project
    artifacts.
  * **Module integrator:** Implement and validate the module integrity
    mechanism.

* **Operational environment**

  * **Reusable components:** Record environments used for project testing.
  * **Module integrator:** Define and test every claimed module operational
    environment.

* **Sensitive security parameters**

  * **Reusable components:** Document algorithm state and possible sensitivity.
  * **Module integrator:** Classify, protect, store, enter, output, and
    zeroize module SSPs.

* **Entropy and random-bit generation**

  * **Reusable components:** Document any algorithm dependencies.
  * **Module integrator:** Provide validated entropy and random-bit
    generation behavior.

* **Physical security**

  * **Reusable components:** Make no general claim.
  * **Module integrator:** Satisfy device-specific physical-security
    requirements.

* **Error state and output inhibition**

  * **Reusable components:** Provide explicit component errors where applicable.
  * **Module integrator:** Enforce module-wide error and output-inhibition
    behavior, including restrictions on externally visible mailbox contents.

* **Installation and operation**

  * **Reusable components:** Provide component integration guidance.
  * **Module integrator:** Provide module installation, initialization,
    administration, and user guidance.

* **CMVP submission**

  * **Reusable components:** Supply relevant component evidence.
  * **Module integrator:** Contract with a laboratory and own the module
    submission.


This division may be refined for a particular integration, but any transfer of
responsibility must be explicit and documented.

## Development Tests and Module Self-Tests

Project development tests and FIPS 140-3 cryptographic algorithm self-tests
serve different purposes.

Development tests may use external test harnesses, dynamically supplied test
vectors, multiple reference implementations, diagnostic output, and test-only
fault injection.

A module cryptographic algorithm self-test must be incorporated into the
module's design and must satisfy the requirements applicable to that module,
including invocation timing, internal determination of pass or fail, error
state behavior, output inhibition, and operator-initiated or periodic
execution where required.

A test used during project development is not a module self-test merely
because it uses a known-answer vector.

## Implementation Variants

Each distinct implementation variant increases the review, testing,
self-test, and validation burden. Variants include, but are not limited to:

* portable-language implementations;
* architecture-specific implementations;
* assembly implementations;
* SIMD implementations;
* hardware-accelerated implementations;
* implementations produced by materially different feature selections; and
* distinct compiled copies residing in different binary artifacts.

The project will initially prefer a single, portable, safe Rust
implementation unless a documented requirement justifies an additional
variant.

Each added variant must be separately identified, analyzed, tested, and
included in the traceability and change-impact records.

## Build and Release Identification

A source revision alone does not uniquely identify a compiled cryptographic
implementation. Assurance records for an identified artifact should include,
as applicable:

* source commit and signed release tag;
* Rust toolchain version;
* compilation target;
* target specification;
* linker and linker version;
* compilation and linker flags;
* enabled features;
* dependency versions;
* relevant environment and configuration inputs;
* produced artifact name, size, and cryptographic digest; and
* test and analysis results associated with that artifact.

A project release identifies a source and evidence baseline. It does not imply
CAVP or CMVP validation.

## Security Claims

Security claims must be precise, scoped, and supported by identified evidence.

The project will not describe an implementation as constant-time,
side-channel-resistant, fault-resistant, formally verified, audited,
validated, approved, or compliant unless the claimed property, applicable
environment, verification method, and limitations are documented.

Absence of an identified defect is not evidence that no defect exists.

## Change Control

Changes to architecture, algorithms, interfaces, security-relevant
implementations, requirements and assurance evidence require documented
impact review appropriate to their scope. The detailed process and source
maintenance rules are recorded in
[configuration-management.md](configuration-management.md), which is under
design review. A document's presence in the repository or a successful CI
run does not establish implementation completion or validation.

Changes to this assurance scope are reviewed through the repository's
branch and pull-request process.
