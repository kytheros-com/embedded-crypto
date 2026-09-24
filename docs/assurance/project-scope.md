# Project Assurance Scope

## Purpose

The `embedded-crypto` project develops reusable cryptographic algorithm
implementations in Rust, together with technical evidence intended to support
their review, integration, testing, and potential use in cryptographic modules
submitted for third-party validation.

The project is designed to make the algorithm implementations and their
supporting evidence understandable and usable by:

* cryptographic module developers;
* security reviewers;
* testing laboratories;
* system integrators; and
* other technically qualified users.

Planned algorithm coverage is recorded in the project standards register. An
algorithm, parameter set, or operation is not considered within the supported
scope until its applicable specifications, parameter sets, operations, and
intended uses have been identified there.

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

This repository therefore provides validation-supporting algorithm components
and evidence. It does not define a universally validated cryptographic module.

A client seeking FIPS 140-3 validation remains responsible for working with an
accredited Cryptographic and Security Testing Laboratory to determine the
applicable requirements and the testing, evidence, and submission procedures
for the client's specific module.

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

* **Algorithm specification**

  * **This project:** Identify and trace applicable requirements.
  * **Integrating module:** Confirm that selected algorithms and uses are
    permitted.

* **Algorithm implementation**

  * **This project:** Provide reviewed source and identified release
    artifacts.
  * **Integrating module:** Incorporate an exact implementation into the
    module.

* **Development testing**

  * **This project:** Provide documented automated tests.
  * **Integrating module:** Evaluate applicability and perform integration
    testing.

* **CAVP testing support**

  * **This project:** Provide test adapters and capability information where
    practical.
  * **Integrating module:** Obtain official testing through an authorized
    laboratory.

* **Cryptographic algorithm self-tests**

  * **This project:** Provide vectors, callable implementation paths, and
    integration guidance.
  * **Integrating module:** Control invocation, pass/fail determination,
    service gating, and failure response.

* **Cryptographic boundary**

  * **This project:** Describe component boundaries and dependencies.
  * **Integrating module:** Define and document the validated module
    boundary.

* **Approved mode**

  * **This project:** Avoid making an independent approved-mode claim.
  * **Integrating module:** Define and enforce approved operation.

* **Roles and services**

  * **This project:** Describe algorithm operations exposed by the
    component.
  * **Integrating module:** Define module roles, services, authorization, and
    authentication.

* **Service indication**

  * **This project:** Document algorithm success and failure behavior.
  * **Integrating module:** Provide the required approved-service
    indication.

* **Module state machine**

  * **This project:** Document component state transitions.
  * **Integrating module:** Define and enforce the module finite-state
    model.

* **Software or firmware integrity**

  * **This project:** Support deterministic identification of project
    artifacts.
  * **Integrating module:** Implement and validate the module integrity
    mechanism.

* **Operational environment**

  * **This project:** Record environments used for project testing.
  * **Integrating module:** Define and test every claimed module operational
    environment.

* **Sensitive security parameters**

  * **This project:** Document algorithm state and possible sensitivity.
  * **Integrating module:** Classify, protect, store, enter, output, and
    zeroize module SSPs.

* **Entropy and random-bit generation**

  * **This project:** Document any algorithm dependencies.
  * **Integrating module:** Provide validated entropy and random-bit
    generation behavior.

* **Physical security**

  * **This project:** Make no general claim.
  * **Integrating module:** Satisfy device-specific physical-security
    requirements.

* **Error state and output inhibition**

  * **This project:** Provide explicit component errors where applicable.
  * **Integrating module:** Enforce module-wide error behavior and output
    inhibition.

* **Installation and operation**

  * **This project:** Provide component integration guidance.
  * **Integrating module:** Provide module installation, initialization,
    administration, and user guidance.

* **CMVP submission**

  * **This project:** Supply relevant component evidence.
  * **Integrating module:** Contract with a laboratory and own the module
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

Security-relevant changes require an assessment of their effect on:

* applicable specification and assurance requirements;
* supported operations and parameter sets;
* public and internal interfaces;
* algorithm behavior;
* implementation variants;
* tests and expected results;
* self-test integration;
* timing and memory behavior;
* operational environments;
* build artifacts;
* prior review evidence; and
* potential CAVP or CMVP testing.

Changes to this assurance scope require review through the repository's normal
pull-request process.
