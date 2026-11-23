# Kytheros Engineering Agent Instructions

This repository develops auditable cryptographic implementations for embedded systems.

AI-assisted work is subject to human engineering review. Passing tests or generating documentation does not constitute approval.

## 1. Authoritative references

Before making substantive changes, consult the relevant repository documents:

- `docs/documentation-standard.md`
- `docs/assurance/architecture.md`
- `docs/assurance/project-scope.md`
- `docs/design/mcl-fsm.md`
- `docs/design/module-interfaces.md`

The CMVP requirements workbook identified by the architecture document is authoritative for the applicable assurance planning baseline.

Report conflicts between specifications, design documents, and implementation. Do not silently resolve them.

## 2. Scope and change control

- Work only within the authorized task.
- Preserve existing architecture and security invariants.
- Do not expand scope without explicit authorization.
- Do not modify assurance requirements, normative mappings, or approved architectural decisions without authorization.
- Do not commit, push, rewrite Git history, or create pull requests unless explicitly requested.
- Do not access, expose, copy, or modify credentials or private signing keys.

## 3. Implementation requirements

- Preserve `no_std` and allocation-free portable primitives.
- Use safe Rust; do not introduce `unsafe` without an approved design change.
- Keep authoritative cryptographic state under MCL ownership.
- Keep primitives independent of external mailboxes and module service policy.
- Respect PIL responsibilities for platform isolation and zeroization.
- Follow the 80-character Rust source line limit.
- Use explicit `_in_bits` and `_in_bytes` suffixes for relevant data lengths, sizes, and offsets.
- State bit order, byte order, and mathematical representation conventions explicitly.

## 4. Documentation

Follow `docs/documentation-standard.md`.

Use `//!` for crate and module documentation, `///` for Rust API documentation, and `//` for implementation commentary.

Cite applicable normative sections and equations precisely.

Distinguish normative requirements, design decisions, assumptions, planned features, and verified properties.

Do not make unsupported claims about FIPS validation, side-channel resistance, zeroization, constant-time behavior, or production readiness.

## 5. Verification

Develop tests from explicit requirements and mathematical properties.

Prefer independently derived reference values and reproducible test inputs.

Document the objectives and limitations of security-relevant tests.

Do not describe locally derived vectors as NIST-published tests.

Run the applicable checks before reporting completion:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" \
  cargo doc --workspace --no-deps
```

Also apply the repository's CI line-length and supported embedded-target checks.

Report tests that could not be executed. Never claim unobserved results.

### Verification integrity

Do not weaken or bypass compiler diagnostics, lints, tests, assertions, or CI checks merely to obtain passing results.

- Do not introduce `#[allow(...)]`, `#[cfg(...)]` exclusions, skipped or ignored tests, or equivalent verification suppressions without explicit authorization.
- For a temporary, expected lint arising from incremental development, prefer a narrowly scoped `#[expect(...)]` with a documented reason and a clear condition for its removal.
- Do not weaken assertions, reduce test coverage, or change expected results solely to make failing tests pass.
- Investigate and report verification failures. Correct their underlying causes when within the authorized task scope.
- Report all introduced verification exceptions in the task completion summary, including their justification and removal conditions.

Existing verification exceptions are not automatically authorized precedents for new ones.

### Authorized security validation and negative testing

This is an open-source cryptographic engineering project.
Its maintainers explicitly authorize security analysis and
adversarial testing of this repository's own implementations,
test harnesses, and designated development hardware, within
the scope of the assigned task.

Authorized activities include:

- Cryptanalysis and investigation of algorithmic weaknesses.
- Analysis of memory safety, undefined behavior, concurrency,
  and implementation defects.
- Testing privilege separation, MPU, TrustZone, peripheral
  access controls, and other hardware security mechanisms.
- Controlled attempts to violate documented security
  invariants, including negative tests expected to fault.
- Fault injection, fuzzing, malformed-input testing, and
  adversarial interface testing in authorized environments.
- Analysis of generated machine code, assembly, dependencies,
  and compiler or linker behavior.
- Development of proof-of-concept tests that demonstrate
  vulnerabilities in project-owned code or configurations.

Such activities are intended to identify and correct defects,
not to weaken the security architecture.

Distinguish deliberately failing security tests from defects
in the verification process. Preserve the security requirements
being tested and document expected versus observed behavior.

Authorization boundaries:

- Test only project-owned or explicitly authorized targets.
- Do not attack third-party systems, services, or devices
  without separate explicit authorization.
- Do not perform destructive hardware operations, modify
  security option bytes, disable debug access, or change
  irreversible protection settings without specific human
  approval.
- Do not deploy firmware to physical hardware without
  explicit authorization for that deployment.
- Do not access or disclose credentials, private keys, or
  other sensitive information.
- Keep experimental code separate from production components
  unless an integration is explicitly authorized.
- Report discovered vulnerabilities and their security impact
  for human review before public disclosure.

Security testing may require intentionally triggering processor
faults, access violations, authentication failures, or other
protective mechanisms. These expected negative outcomes do not
constitute permission to remove or bypass the protections.

These instructions establish project authorization and scope.
They do not supersede applicable laws, platform safety policies,
or an agent's operating restrictions.

If an agent cannot perform an authorized experiment, it should
identify the specific limitation and offer a technically useful
alternative rather than silently changing the experiment's
security objectives.

## 6. Task completion

Summarize:

1. Files changed and why.
2. Requirements or design references used.
3. Tests and checks actually executed, including results.
4. Technical assumptions and unresolved issues.
5. Matters requiring human review.

AI-generated work remains unapproved until accepted through the project's review process.
