# Reference Module Logical Interface Contract

**Status:** Architectural baseline; transfer protocol under design  
**Date:** 2026-10-22  
**Scope:** Embedded reference integration, not standalone primitive crates

## 1. Architectural boundary

The Module Control Layer (MCL) exclusively mediates the reference module's
logical **service-level** information flow. The Platform Integration Layer
(PIL) supplies and enforces memory access, atomic synchronization and
target-specific reset/fault entry mechanisms. Cryptographic primitives only
access explicitly authorized **internal** storage; neither external callers
nor primitives directly share internal primitive storage with the five
interface payload mailboxes.

The logical service interface is not automatically the formally defined
FIPS 140-3 cryptographic boundary or physical perimeter. Its exact embodiment,
ports and operational environment remain open in workbook question Q-001.
The five categories derive from ISO/IEC 19790:2012 §7.3.3 (AS03.01–AS03.15)
and ISO/IEC 24759:2017 §6.3. Physical power and processor reset/exception
entry are separately accounted for by the PIL. No trusted channel or trusted
path is included in the initial reference-module scope.

## 2. Five distinct fixed-memory mailbox payloads

The reference integration shall have **exactly five separate fixed-memory
payload regions** implementing the mandatory logical interface categories.
Addresses, region sizes, alignment and physical protection depend on the
selected PIL, not on the portable MCL.

| Mailbox | Information category | MCL payload permission | External payload permission | Primitive permission |
| --- | --- | --- | --- | --- |
| **Control input** | Service requests, operation selection and inbound protocol events | RO | RW | None |
| **Control output** | Outbound acknowledgments, control commands and protocol events | RW | RO | None |
| **Status output** | Module/service status, self-test and error indicators, approved-service indication | RW | RO | None |
| **Data input** | Service input data, including authorized plaintext, ciphertext, keys/SSPs, authentication and public data | RO | RW | None |
| **Data output** | Authorized service results such as digests, signatures, public keys, ciphertext and permitted ordinary plaintext | RW | RO | None |

RO means read-only and RW means read/write. These are the normal **payload
access permissions**, which remain in effect regardless of mailbox ownership
or semaphore state. Primitive permission **None** excludes reads, writes and
execution. The PIL may perform narrowly scoped privileged
maintenance/zeroization operations outside the normal MCL/external payload
permissions. A target shall not weaken this matrix to accommodate a device.
The MCL does not rely on external clients to obey the semaphore protocol.

Only the MCL copies between these externally addressable payloads and
protected internal storage, after authorization and validation. No external
caller-supplied pointer may reach a primitive, and a primitive never receives
a mailbox pointer. Primitive crates themselves remain platform-independent,
using documented internal caller-provided memory contracts.

## 3. Five independent atomic synchronization objects

Each payload mailbox has an **associated atomic semaphore or equivalent
ownership/synchronization object**. Its location and permissions are
**independent** of the mailbox payload's read/write permissions. The object
must support atomic authorized producer/consumer transitions and appropriate
acquire/release ordering and visibility guarantees.

Control mailboxes may carry protocol events such as **message available**,
**message accepted**, **message rejected**, **operation completed** and
**output consumed**. The MCL need not write into an inbound payload to
acknowledge it: MCL acknowledgments travel via control output and/or
authorized semaphore transitions. External acknowledgments travel via
control input and/or authorized semaphore transitions.

**Adversarial-client model:** The external caller, including the orchestrating
host, may be compromised. It may supply arbitrary data, violate the handshake,
repeatedly change input bytes, read output before notification, reuse stale
messages, or decline to consume results. Semaphores define *protocol
ownership and validity* for cooperating participants; they neither
authenticate the client nor constrain its physical payload permissions.

**Input acceptance:** On a properly recognized request event, the MCL shall
copy a bounded candidate request and data payload into protected internal
storage. It shall perform all security-relevant validation on that internal
copy (including message structure, bounds, service parameters, role/state
authorization and applicable SSP-entry policy), and dispatch primitives
using **only** the validated internal representation. The MCL shall not
re-read mutable external fields to govern an operation already accepted.
An adversarial client can alter its mailbox *during* the copy; this may
produce a mixed or inconsistent candidate. The MCL is not required to
reconstruct the client's intended message or establish that all bytes
existed simultaneously. Instead it must treat the resulting snapshot as
arbitrary untrusted bytes, perform bounds-safe acquisition and validation,
and reject any candidate that violates the service contract. A
semantically valid but adversarially chosen candidate is still untrusted;
the MCL must remain secure when processing it. An invalid request is
normally a specified service rejection, not an unexpected primitive fault.

**Output publication:** The MCL verifies operation success, authorization,
security classification and the current module state **before copying**
a result into an externally readable output mailbox. It shall never place
plaintext CSPs, unauthorized SSPs or other prohibited information into
mailbox payloads, even temporarily. The MCL then copies the authorized
result and **publishes** the message only by completing the specified
atomic semaphore transition with the required memory ordering. Before
that transition, a client's read has **no valid message semantics**:
partial or stale bytes are not an accepted module result. The module
does not promise coherent early reads or prevent subsequent alteration
or misuse of information by an external recipient.

**Normative caveat:** These publication semantics do **not** waive the
requirements to inhibit data output and, as applicable, control output
during specified self-test, manual SSP-entry, loading, zeroization and
error conditions (ISO/IEC 19790 AS03.07 and AS03.10 and applicable CMVP
guidance). An externally readable memory region may constitute a data
output path even when a semaphore says "not published". The actual
mailbox placement, write behavior, stale-content sanitization, gating
and validation evidence must demonstrate those requirements. Any
remaining feasibility/compliance issue is an **open design gate**, not
silently resolved by redefining publication.

## 4. Required information-flow policy

- Control input: commands, service selections, data-ready and caller
  acknowledgment events, subject to roles, valid FSM state and message checks.
- Data input: untrusted, bounded payloads. The MCL copies a candidate into
  internal storage and validates **that internal copy** before primitive use;
  it classifies all SSP and authentication inputs.
- Control output: allowed MCL-originated controls and acknowledgments; it
  shall not serve as a covert data-output path.
- Status output: defined module/service status and approved/error indicators;
  it shall not reveal SSPs or bypass data-output inhibition.
- Data output: after operation completion, authorization, sensitivity and
  state checks, the MCL copies only permitted results into the output mailbox;
  the message becomes **published** upon the defined atomic transition.

The output-inhibition obligations of ISO/IEC 19790 AS03.07 and AS03.10
apply independently of whether the caller obeys mailbox semaphores.
Unexpected primitive internal faults trigger zeroization of **every memory
region the primitive was capable of modifying**. Mailbox payloads and
in-flight transfers, which primitives cannot access, must be separately
invalidated or sanitized according to the module-wide SSP/output policy.

Hardware-originating faults, resets, interrupts and power events are
handled at the appropriate PIL entry points and presented internally to
the MCL; their classification relative to the *formal* module boundary
is a separate Q-001 decision. They are not hidden service-data mailboxes.

### 4.1 Scope of SSP import and protected export

For the current Level 1 reference design, plaintext SSP entry remains
permitted *subject to the specific SSP/service contract*, but plaintext
CSP export is prohibited by project policy. Encrypted SSP backup does not
guarantee confidentiality against a compromised host that can select or
learn the password used to derive its wrapping key. This is a documented
operational/threat-model limitation, **not** a claim that FIPS Level 1
necessarily requires such exposure. An independently authorized recovery
authority or trusted recipient credential infrastructure is outside the
current reference scope.

## 5. Portability and evidence

The target-independent PIL contract requires fixed bounded regions,
hardware-enforced least-privilege permissions, effective primitive
exclusion from mailboxes/MCL state, restrictions on relevant DMA and other
bus masters, trustworthy region write-capability accounting, fault handoff,
zeroization and fail-secure initialization. The STM32U5A5ZJ's documented
MPU/TrustZone/GTZC/exception capabilities are a provisional implementation
basis; hardware enforcement awaits integration testing. A hardware board is
**not required before design work continues**.

ISO/IEC 24759 VE/TE03.01–03.15 requires testable descriptions of each
physical/logical path, category separation, information formats, validators,
output behavior and state-dependent inhibition. These are design evidence
obligations; mailbox plans are not implementation proof.

## 6. Deferred engineering details

Unsettled: actual addresses and sizes; data/control/status record fields;
number of outstanding requests; semaphore representation and privileges;
memory-ordering implementation; bounds-safe snapshot and validation rules;
publication sequencing and the CMVP-compliant treatment of physically
readable output during required inhibition; fault/reset transaction recovery;
timeouts; message framing and maximum lengths; checksums and control/status
error-detecting codes. Unkeyed checksums/EDCs can detect some accidental
corruption but do not authenticate a malicious sender.

The workbook remains authoritative for requirement, decision, question,
service and evidence identifiers. This accepted architecture must be mapped
into the workbook through controlled review; no permanent new workbook
identifiers are invented in this document. The FSM constraints research is
preserved at [fsm-requirements-notes.md](../assurance/fsm-requirements-notes.md).
