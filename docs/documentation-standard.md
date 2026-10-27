# Kytheros Engineering Documentation Standard

**Version:** 0.1  
**Status:** Proposed engineering standard  
**Scope:** Embedded-crypto repository  
**Applicability:** Human-authored and AI-assisted engineering work

## 1. Purpose

This standard establishes the requirements for documenting the design, implementation, verification, and assurance of Kytheros embedded cryptographic software.

Documentation is an engineering deliverable. It shall be sufficiently precise that a technically qualified reviewer can determine:

1. What the component is required to do.
2. How its implementation satisfies those requirements.
3. What assumptions and constraints apply.
4. What evidence supports its correctness.
5. What remains unimplemented, untested, or unverified.

Documentation shall support independent review without requiring access to the original author.

## 2. Governing principles

### 2.1 Correctness and traceability

Documentation shall accurately reflect the implementation and distinguish normative requirements from Kytheros design decisions.

Claims shall be traceable to their supporting specifications, designs, analyses, tests, or other evidence.

Unresolved questions shall be recorded explicitly rather than resolved through unsupported assumptions.

### 2.2 Automated enforcement

**Every mandatory, mechanically verifiable convention shall have an automated check.**

Exceptions shall be explicit, justified, narrowly scoped, and reviewable.

A convention without an automated check shall be identified as manually verified or as requiring additional tooling.

Passing automated checks does not establish mathematical correctness, security, compliance, or fitness for production.

### 2.3 Human technical review

Security-relevant documentation and implementation shall receive human review appropriate to their assurance significance.

Automated tests and AI-generated explanations may assist review but do not replace independent engineering judgment.

### 2.4 Accuracy over presentation

Precise and concise explanations are preferred over extensive boilerplate.

Comments shall explain contracts, invariants, mathematical reasoning, security assumptions, and nonobvious implementation decisions.

Comments that merely restate obvious source-code operations should generally be avoided.

## 3. Document hierarchy and authority

The repository uses the following documentation hierarchy.

| Level | Contents | Principal location |
|---|---|---|
| Requirements and assurance | Normative applicability, requirements, evidence plans, decisions and open questions | `docs/assurance/` |
| Architecture and detailed design | Component responsibilities, interfaces, state machines and design rationale | `docs/assurance/` and `docs/design/` |
| Development standards | Engineering conventions, methods and review requirements | `docs/development/` |
| Crate and module documentation | Purpose, scope, integration contracts and limitations | Rust `//!` comments |
| API documentation | Behavioral contracts, equations, inputs and outputs | Rust `///` comments |
| Implementation commentary | Local invariants and nonobvious implementation details | Rust `//` comments |
| Verification evidence | Test objectives, vectors, derivations, coverage and results | Source tests and assurance records |

The CMVP requirements workbook identified in `docs/assurance/architecture.md` is the controlling planning record for the applicable assurance requirements, design decisions, open questions, and evidence identifiers.

This documentation standard does not amend those requirements or architectural decisions.

Conflicting documents shall be reported for resolution. An implementation comment shall not silently override an authoritative requirement.

## 4. Source-code documentation

### 4.1 Rust documentation conventions

Use conventional Rust documentation syntax:

- `//!` for crate and module documentation.
- `///` for public items and appropriate internal items.
- `//` for implementation commentary that need not appear in generated API documentation.

Rustdoc comments shall use valid Markdown and resolvable links.

Executable Rustdoc examples should be provided where they materially clarify an interface or contract.

### 4.2 Crate and module documentation

Crate and module documentation shall describe, as applicable:

- Purpose and current implementation scope.
- Applicable algorithm or engineering specifications.
- Relevant decomposition into components.
- Data representations and conventions.
- Dependencies and environmental assumptions.
- State ownership and integration boundaries.
- Working-memory and allocation constraints.
- Known limitations and assurance status.
- References to relevant design documents.

Documentation shall not describe planned functionality as implemented functionality.

### 4.3 Function and type documentation

Security-relevant interfaces shall document, where applicable:

- The function's purpose.
- Input and output meanings.
- Data types, lengths, units, ranges and alignment.
- Preconditions and postconditions.
- Invariants and mathematical properties.
- Error, panic and failure behavior.
- State modifications and side effects.
- Ownership and lifetime requirements.
- Relevant memory-clearing responsibilities.
- Normative specification references.
- Verification assumptions and limitations.

Trivial internal helpers may use shorter documentation when their behavior is evident and their contract is local.

Documentation detail shall be proportional to security significance and conceptual complexity.

### 4.4 Mathematics and normative specifications

Implementations of standardized cryptographic algorithms shall cite the applicable publication, revision, section, and equation or algorithm step where available.

Mathematical definitions shall be represented accurately and preserve recognizable notation from the source specification.

For example:

`Ch(x,y,z) = (x AND y) XOR (NOT x AND z)`

Reference: NIST FIPS 180-4, §4.1.2, Eq. (4.2).

Do not claim that a locally derived vector or mathematical identity was published by NIST.

## 5. Representation and naming conventions

### 5.1 Lengths and units

Identifiers representing lengths, sizes, offsets, and capacities measured in bits or bytes shall include the suffix `_in_bits` or `_in_bytes`.

Examples:

- `message_len_in_bits`
- `buffer_len_in_bytes`
- `offset_in_bits`
- `capacity_in_bytes`

Constants may use corresponding uppercase suffixes, such as `BLOCK_SIZE_IN_BITS`.

Identifiers representing words, blocks, elements, or indices shall explicitly identify those units or roles where ambiguity is possible.

Conversions between bits and bytes shall be explicit and checked for overflow as appropriate.

### 5.2 Bit and byte order

Documentation shall specify relevant representation conventions, including:

- Bit numbering and most/least significant bit.
- Ordering of bits in partially filled bytes.
- Byte ordering within multibyte words.
- Message and output serialization order.
- Bit-length encoding and padding conventions.

SHA-2 implementations shall follow the applicable conventions of FIPS 180-4.

No host-endianness assumption shall be implicit in portable algorithm code.

### 5.3 Matrix and vector conventions

Documentation involving matrices shall specify:

- The underlying field or ring.
- Matrix dimensions.
- Row-vector or column-vector multiplication.
- Bit ordering within encoded vectors.
- The correspondence between stored words and matrix rows or columns.

For the existing SHA-256 inverse test matrices:

`y = x * A` over GF(2), with row vectors.

Array entry `a[i]` encodes row `i`; bit `j` represents matrix entry `A[i,j]`.

A basis word `1 << i` selects row `i`.

These conventions shall be preserved unless a later design decision explicitly changes them.

### 5.4 Formatting

Rust source files shall follow the repository's `rustfmt.toml`.

The maximum line length for Rust source files is 80 characters, subject to explicitly approved exceptions.

`cargo fmt --check` alone does not enforce this limit; the separate CI line-length check is authoritative for that mechanical constraint.

Comments and documentation shall be written for readability rather than compressed merely to satisfy formatting checks.

## 6. Architecture and security contracts

### 6.1 Component responsibilities

The established architectural separation shall be preserved:

- **Cryptographic primitives:** Implement algorithmic behavior using authorized working storage.
- **MCL:** Owns authoritative cryptographic state, service policy, context lifecycle, and successful state commits.
- **PIL:** Provides platform-dependent isolation, hardware operations, and zeroization mechanisms.

Source documentation shall distinguish planned integration contracts from implemented protections.

### 6.2 Memory and state ownership

Security-relevant APIs shall identify the owner of authoritative state and any temporary working copies.

Documentation shall distinguish logical invalidation, Rust object lifetime, compiler-generated temporaries, and physical memory zeroization.

No secure-erasure guarantee shall be inferred solely from assignment, scope exit, ownership transfer, or the use of `Drop`.

### 6.3 Portability and execution

Portable cryptographic primitives shall preserve the established `no_std`, allocation-free, and safe-Rust requirements unless an authorized design decision provides otherwise.

Target-specific code and unavoidable `unsafe` operations shall have explicit justification, containment, review requirements, and supporting evidence.

### 6.4 Unsupported claims

Documentation shall not claim any of the following without separately identified supporting evidence:

- FIPS 140-3 or CMVP module validation.
- CAVP/ACVP algorithm validation.
- Constant-time execution.
- Side-channel resistance.
- Fault resistance.
- Complete or reliable zeroization.
- Formal verification.
- Independent audit or production readiness.

These properties are distinct and shall not be inferred from successful functional testing.

## 7. Test documentation and verification evidence

### 7.1 Test objectives

Security-relevant tests shall identify the mathematical, functional, or security property they are intended to verify.

Tests should be structured to provide meaningful evidence, not merely a large count of assertions.

Where practical, document:

- Test objective and expected behavior.
- Specification or requirement being verified.
- Origin of test inputs and expected outputs.
- Construction of independent reference values.
- Coverage rationale and limitations.
- Relevant execution environment.

### 7.2 Test-vector provenance

Distinguish among:

1. Authoritative published known-answer tests.
2. Independently derived mathematical reference values.
3. Locally constructed deterministic test vectors.
4. Generated or sampled test inputs.

Reference values shall not be described as independent merely because they are calculated in a separate function.

Where feasible, preserve or document reproducible generation methods and versioned supporting scripts.

### 7.3 Mathematical properties

Tests should exploit the mathematical structure of the function under examination.

Relevant approaches may include:

- Boolean truth tables and positional independence.
- Basis-vector testing of linear transformations.
- Linearity and other algebraic identities.
- Independently constructed inverse transformations.
- Boundary-condition tests.
- Deterministic generators with documented state transitions.
- Published intermediate algorithm values.
- Differential testing against independent implementations.

Mathematical assumptions shall be stated.

For example, agreement on a basis establishes agreement throughout a vector space only when the relevant transformations are linear.

Sampled observations shall not be described as exhaustive proofs.

### 7.4 Deterministic test generators

Document the recurrence, polynomial or other defining parameters, state width, seed, output convention, and intended coverage.

For LFSRs, distinguish feedback direction, Galois versus Fibonacci realization, reciprocal-polynomial conventions, and the assumptions supporting period claims.

### 7.5 Test implementation choices

Nonobvious optimizations shall be explained when they make tests harder to understand.

Examples include branchless bit-mask construction and accumulating mismatch bits with bitwise OR.

Tests should provide useful diagnostics when practical, but performance-sensitive test infrastructure may use alternative approaches when their correctness and rationale are documented.

### 7.6 Evidence limitations

Documentation shall distinguish:

- Tests passing in a development environment.
- Tests passing on an embedded target.
- Mathematical or formal assurance.
- Algorithm-validation testing.
- Module-level security assurance.

The absence of detected failures is not proof of complete correctness.

## 8. Documentation lifecycle

Documentation shall be updated in the same change as any implementation or design modification that makes it inaccurate.

A change review shall consider impacts on requirements, architecture, interfaces, source documentation, tests, evidence, and assurance claims.

Normative source references, requirement identifiers, and approved decisions shall not be silently replaced or renumbered.

Historical review baselines shall be labeled as historical. Current implementation status shall not be inferred from an outdated working-branch reference.

AI-generated documentation is a draft until reviewed and accepted by the responsible engineer.

## 9. Automated checks and exceptions

Required automated checks for the current Rust workspace include:

- `cargo fmt --all --check`
- Repository CI Rust source line-length check.
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --all-targets --locked`
- `cargo test --doc --workspace`
- Rustdoc generation with broken intra-document links treated as errors.
- Existing supported bare-metal target checks.

Checks not yet implemented in CI shall remain documented as verification requirements, not be described as existing CI enforcement.

Approved exceptions shall identify the requirement, affected files or components, justification, reviewer, and mechanism used to preserve reviewability.

Warnings shall not be suppressed merely to achieve a successful build.

## 10. Review criteria

A documentation change is ready for acceptance when:

1. Its technical claims are correct and consistent with authoritative sources.
2. Its contracts and limitations are unambiguous.
3. Its references and mathematical conventions are accurate.
4. Its executable examples and relevant automated checks pass.
5. Its verification claims are supported by appropriate evidence.
6. Relevant architecture and requirements impacts have been considered.
7. Outstanding issues are recorded rather than concealed.

The responsible engineer retains approval authority for all security-relevant documentation and assurance claims.
