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

Follow `docs/development/documentation-standard.md`.

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

## 6. Task completion

Summarize:

1. Files changed and why.
2. Requirements or design references used.
3. Tests and checks actually executed, including results.
4. Technical assumptions and unresolved issues.
5. Matters requiring human review.

AI-generated work remains unapproved until accepted through the project's review process.
