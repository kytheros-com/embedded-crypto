# Standards Register

## Purpose

This register identifies the external publications used to define, interpret,
test, or constrain this project's cryptographic implementations and assurance
evidence.

The register does not reproduce requirements from copyrighted publications.
It records source identity, revision, role, review state, and known issues.
Detailed requirements will be recorded separately and will cite the source
identifiers assigned here.

Inclusion in this register does not imply that this library or any integrating
module has been validated under FIPS 140-3 or any other program.

## Register state

* Last reviewed: 2026-09-24
* Review scope: Initial FIPS 140-3 framework, SHA-2, and CNSA 2.0 sources
* Validation target: Supporting evidence for future integrating modules
* Validation authority: Not selected
* Cryptographic Security Testing laboratory: Not selected

## Source roles

* **Framework**

  * Defines cryptographic-module security or testing requirements.

* **Program guidance**

  * Defines CMVP interpretations, procedures, or administrative expectations.
  * May change without a new edition of the underlying standard.

* **Algorithm specification**

  * Defines a cryptographic algorithm or its permitted parameters.

* **Validation specification**

  * Defines algorithm-validation protocols, schemas, or test procedures.

* **Deployment profile**

  * Selects algorithms or parameters for a particular operational context.

* **Monitor**

  * May supersede or affect a project source, but its applicability has not
    yet been established.

## Review states

* **Metadata verified**

  * The title, issuer, revision, and official source have been checked.

* **Acquisition pending**

  * A lawfully licensed complete copy has not been recorded as available.

* **Requirements extraction pending**

  * Applicable requirements have not yet been entered into the requirements
    matrix.

* **Requirements extracted**

  * Applicable requirements have been entered into the requirements matrix.

* **Traceability reviewed**

  * Requirement mappings have received an independent review.

## Maintenance rules

* A source identifier shall never be reused for a different publication.
* A revision change shall be recorded explicitly.
* Requirements shall cite both a source identifier and a clause, section, or
  other stable location within that source.
* Mutable web guidance shall include a retrieval date.
* When a mutable source is used as evidence, the exact retrieved artifact and
  its cryptographic digest shall be retained in the project evidence archive.
* Licensed publications shall not be committed to the public repository unless
  their licenses expressly permit redistribution.
* Source changes shall be evaluated for effects on requirements, design,
  implementation, tests, and assurance evidence.
* Unresolved applicability questions shall be recorded rather than resolved by
  undocumented assumption.

## FIPS 140-3 framework sources

### SR-FIPS-140-3

* Document:

  * FIPS PUB 140-3, *Security Requirements for Cryptographic Modules*
* Issuer:

  * National Institute of Standards and Technology
* Publication date:

  * 2019-03-22
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.FIPS.140-3
* Notes:

  * The publication incorporates ISO/IEC cryptographic-module standards into
    the United States and Canadian validation program.
  * It does not by itself contain all requirements and program guidance needed
    for a validation submission.

### SR-ISO-19790-2012

* Document:

  * ISO/IEC 19790:2012, including Cor.1:2015,
    *Security requirements for cryptographic modules*
* Issuer:

  * International Organization for Standardization
  * International Electrotechnical Commission
* Edition:

  * Edition 2
  * 2012, incorporating the 2015 technical correction
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Acquisition pending
  * Requirements extraction pending
* Official source:

  * https://www.iso.org/standard/52906.html
* Notes:

  * Current public CMVP materials identify this edition as part of the
    FIPS 140-3 validation framework.
  * ISO now identifies this edition as withdrawn.
  * See `SR-ISSUE-001`.

### SR-ISO-24759-2017

* Document:

  * ISO/IEC 24759:2017,
    *Test requirements for cryptographic modules*
* Issuer:

  * International Organization for Standardization
  * International Electrotechnical Commission
* Edition:

  * Edition 3
  * 2017
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Acquisition pending
  * Requirements extraction pending
* Official source:

  * https://www.iso.org/standard/72515.html
* Notes:

  * Current public CMVP materials identify this edition as part of the
    FIPS 140-3 validation framework.
  * ISO now identifies this edition as withdrawn.
  * See `SR-ISSUE-001`.

## CMVP publications

### SR-SP-800-140

* Document:

  * NIST SP 800-140,
    *FIPS 140-3 Derived Test Requirements*
* Revision:

  * Final, 2020-03-20
* Role:

  * Framework
  * Validation specification
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140

### SR-SP-800-140A

* Document:

  * NIST SP 800-140A,
    *CMVP Documentation Requirements*
* Revision:

  * Final, 2020-03-20
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140A

### SR-SP-800-140B-R1

* Document:

  * NIST SP 800-140B Revision 1,
    *CMVP Security Policy Requirements*
* Revision:

  * Revision 1, 2023-11-17
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140Br1

### SR-SP-800-140C-R2

* Document:

  * NIST SP 800-140C Revision 2,
    *CMVP-Approved Security Functions*
* Revision:

  * Revision 2, 2023-07-25
* Role:

  * Framework
  * Algorithm specification
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140Cr2

### SR-SP-800-140D-R2

* Document:

  * NIST SP 800-140D Revision 2,
    *CMVP-Approved Sensitive Security Parameter Generation and
    Establishment Methods*
* Revision:

  * Revision 2, 2023-07-25
* Role:

  * Framework
  * Algorithm specification
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140Dr2

### SR-SP-800-140E

* Document:

  * NIST SP 800-140E,
    *CMVP Approved Authentication Mechanisms*
* Revision:

  * Final, 2020-03-20
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140E

### SR-SP-800-140F

* Document:

  * NIST SP 800-140F,
    *CMVP Approved Non-Invasive Attack Mitigation Test Metrics*
* Revision:

  * Final, 2020-03-20
* Role:

  * Framework
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.SP.800-140F

## CMVP program guidance

### SR-CMVP-IG

* Document:

  * *Implementation Guidance for FIPS 140-3 and the Cryptographic Module
    Validation Program*
* Revision:

  * 2026-08-19
* Role:

  * Program guidance
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Retrieval date:

  * 2026-09-24
* Official source:

  * https://csrc.nist.gov/projects/cryptographic-module-validation-program/fips-140-3-ig-announcements
* Notes:

  * This is a mutable source.
  * Its revision shall be rechecked before each release that claims an
    assurance-evidence baseline.

### SR-CMVP-MM

* Document:

  * *FIPS 140-3 Cryptographic Module Validation Program Management Manual*
* Revision:

  * Version 2.7
  * 2026-04-09
* Role:

  * Program guidance
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Retrieval date:

  * 2026-09-24
* Official source:

  * https://csrc.nist.gov/projects/cryptographic-module-validation-program/cmvp-fips-140-3-management-manual
* Notes:

  * This document primarily governs validation-program processes,
    responsibilities, and submissions.
  * It remains relevant even where requirements cannot be satisfied by this
    library alone.

### SR-CMVP-RFG

* Document:

  * CMVP Requests for Guidance and CMVP Resolutions
* Revision:

  * Continuously maintained
* Role:

  * Program guidance
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Retrieval date:

  * 2026-09-24
* Official source:

  * https://csrc.nist.gov/projects/cryptographic-module-validation-program/fips-140-3-ig-announcements
* Notes:

  * This is a mutable collection.
  * Applicable resolutions shall be recorded individually when they affect a
    project requirement or decision.

## SHA-2 sources

### SR-FIPS-180-4

* Document:

  * FIPS PUB 180-4,
    *Secure Hash Standard*
* Issuer:

  * National Institute of Standards and Technology
* Revision:

  * Initial publication, 2012-03
  * Updated, 2015-08
* Role:

  * Algorithm specification
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://doi.org/10.6028/NIST.FIPS.180-4
* Notes:

  * This is the primary algorithm specification for the initial
    `kytheros_sha2` implementation.
  * Any subsequent revision or replacement shall receive a distinct register
    entry and a change-impact review.

### SR-ACVP-SHA

* Document:

  * *ACVP Secure Hash Algorithm JSON Specification*
* Issuer:

  * National Institute of Standards and Technology
* Revision:

  * Continuously maintained Internet-Draft publication
* Role:

  * Validation specification
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Retrieval date:

  * 2026-09-24
* Official source:

  * https://pages.nist.gov/ACVP/draft-celi-acvp-sha.html
* Notes:

  * This source defines validation-protocol data structures and test
    processing.
  * It does not replace FIPS 180-4 as the algorithm specification.
  * A specific retrieved revision and digest shall be recorded before ACVP
    evidence is generated.

## CNSA 2.0 policy sources

### SR-CNSSP-15

* Document:

  * CNSS Policy No. 15,
    *Use of Public Standards for Secure Information Sharing*
* Issuer:

  * Committee on National Security Systems
* Revision:

  * Released 2025-03-04
* Role:

  * Deployment profile
* Project state:

  * Metadata verified
  * Acquisition pending
  * Requirements extraction pending
* Official source:

  * https://www.nsa.gov/Cybersecurity/Post-Quantum-Cybersecurity-Resources/
* Notes:

  * The exact acquired publication shall be retained in the evidence archive.
  * Requirements applicable to a reusable algorithm library shall be
    distinguished from requirements imposed on complete systems.

### SR-CNSA-2-FAQ

* Document:

  * *Commercial National Security Algorithm Suite 2.0 FAQ*
* Issuer:

  * National Security Agency
* Revision:

  * Version 2.1
  * 2024-12
  * Document identifiers U/OO/194427-22 and PP-24-4014
* Role:

  * Deployment profile
  * Program guidance
* Project state:

  * Metadata verified
  * Requirements extraction pending
* Official source:

  * https://www.nsa.gov/Press-Room/Digital-Media-Center/Document-Gallery/igphoto/2003071836/
* Notes:

  * This source supplies selection and transition context.
  * It does not replace the applicable NIST algorithm specifications.

## Monitored ISO editions

### SR-ISO-19790-2025

* Document:

  * ISO/IEC 19790:2025,
    *Security requirements for cryptographic modules*
* Issuer:

  * International Organization for Standardization
  * International Electrotechnical Commission
* Edition:

  * Edition 3
  * 2025-02
* Role:

  * Monitor
* Project state:

  * Metadata verified
  * Applicability unresolved
* Official source:

  * https://www.iso.org/standard/82423.html
* Notes:

  * This edition supersedes ISO/IEC 19790:2012 within the ISO catalogue.
  * Its adoption schedule for CMVP submissions has not been established by
    this project.
  * See `SR-ISSUE-001`.

### SR-ISO-24759-2025

* Document:

  * ISO/IEC 24759:2025,
    *Test requirements for cryptographic modules*
* Issuer:

  * International Organization for Standardization
  * International Electrotechnical Commission
* Edition:

  * 2025
* Role:

  * Monitor
* Project state:

  * Metadata verified
  * Applicability unresolved
* Official source:

  * https://www.iso.org/standard/82424.html
* Notes:

  * This edition defines testing for ISO/IEC 19790:2025.
  * Its adoption schedule for CMVP submissions has not been established by
    this project.
  * See `SR-ISSUE-001`.

## Open issues

### SR-ISSUE-001: Applicability of the 2025 ISO editions

* Observation:

  * ISO published new editions of ISO/IEC 19790 and ISO/IEC 24759 in 2025.
  * Current public CMVP materials continue to identify ISO/IEC 19790:2012,
    including Cor.1:2015, and ISO/IEC 24759:2017.
  * No public CMVP transition notice adopting the 2025 editions was located
    during the review performed on 2026-09-24.

* Interim project treatment:

  * Map the current CMVP framework using `SR-ISO-19790-2012` and
    `SR-ISO-24759-2017`.
  * Monitor `SR-ISO-19790-2025` and `SR-ISO-24759-2025`.
  * Do not claim that the interim treatment determines which editions would
    govern a future validation submission.

* Required resolution:

  * Confirm the applicable editions and transition rules with a selected
    Cryptographic Security Testing laboratory or directly with CMVP before
    establishing a validation-submission baseline.

* Potentially affected artifacts:

  * Requirements matrix
  * Evidence plan
  * Security policy
  * Test plan
  * Vendor evidence
  * Test evidence
  * Validation schedule and cost estimate
