# Embedded-Crypto Architecture

**Status:** Draft architecture baseline; conceptual MCL FSM adopted; implementation and assurance evidence remain open  
**Last reviewed:** 2026-10-23  
**Repository:** `kytheros-com/embedded-crypto`  
**Review baseline:** `main` at `6675ba7a434dbf9112c6bc0693249d05771a5f78`  
**Working branch:** `design-requirements-audit/2026-10-19`

## 1. Purpose and scope

This document defines the architecture for the Kytheros `embedded-crypto` project.

The project distinguishes three logical architectural layers:

1. **Cryptographic primitives** implement specified algorithms, expose bounded and explicit interfaces, and supply algorithm-level evidence.
2. The **Platform Integration Layer (PIL)** implements narrowly scoped platform-dependent mechanisms for memory placement, protection and zeroization; hardware access; reset and exception entry; and privilege transitions.
3. The **Module Control Layer (MCL)** controls external module service interfaces, cryptographic dispatch, state transitions, self-test coordination, SSP lifecycle and the selected security policy.

These are logical responsibilities, not necessarily separate crates, binaries, security domains or validation boundaries. The complete cryptographic boundary and module embodiment remain open under Q-001.


## 2. Architectural principles

- **Separation of concerns.** A primitive implements an algorithm, not an entire module security policy. The PIL implements platform mechanisms, not cryptographic-service policy. The MCL orchestrates and enforces security policy.
- **Portable embedded core.** Production primitive paths shall be designed from the outset for `no_std` and without `alloc`, operating before an OS or fully configured MMU exists. A hosted test harness may use `std` without changing the embedded runtime contract.
- **Explicit resource ownership.** Portable primitives use authorized internal caller-provided storage, never external mailbox references or authoritative MCL contexts/key records. Only the MCL commits successful working-copy context updates.
- **Fail-secure bounded state.** Fixed-member composition groups, one-way context retirement, spent IVs, irreversible decryption-only downgrades, and reset invalidation replace rollback and generalized recovery.
- **MCL-enforced logical boundary.** Five separate fixed-memory mailboxes implement logical service information flow. The external caller is potentially adversarial: the MCL validates internal copies of all inbound information, authorizes outbound information before copying, and publishes complete messages through atomic synchronization; the PIL enforces memory isolation. **No outward result byte is written before the entire result is authorized**; the semaphore is cooperative synchronization, not an access-control barrier.
- **Small, reviewable trusted mechanisms.** Prefer safe Rust for portable primitives. Any unavoidable `unsafe` code shall be narrowly contained, justified, reviewed, and tested in the component requiring it; the present workspace-level `unsafe_code = "forbid"` policy must not be weakened globally without an explicit design decision.
- **No claims by association.** Correctness, memory safety, algorithm testing, module validation, side-channel behavior, and physical security are different properties, requiring separately identified evidence.
- **Traceable change control.** Architecture changes are reviewed for effects on design, requirements, interfaces, security behavior, tests, and evidence before being accepted.

## 3. Authoritative records and document relationships

The source of truth for the **CMVP planning baseline** is:

`docs/assurance/CMVP_Level_1_Software_Requirements_Matrix.xlsx`

which we often refer to simply as **The Workbook**. Its relevant worksheets include `Source Register`, `Requirements`, `Design Decisions`, `Open Questions`, `Service Contracts`, `SSP Plan`, `ISO Level 1 Index`, and `Evidence Plan`. Preserve their stable requirement, decision, question, assertion, and evidence identifiers.

- **This document** defines the logical architecture, component responsibilities, interface boundaries, and the design principles that implement recorded decisions.
- **The Workbook** records controlling sources, normative applicability, project requirements, decision status, planned services, evidence ownership and completion state.
- **`docs/assurance/project-scope.md`** defines assurance claims and the division between this project and a third-party integrating module. This architecture refines, but must not contradict, that scope.
- **`docs/design/module-interfaces.md`** records the five-mailbox permissions and open synchronization/visibility questions.
- **`docs/assurance/fsm-requirements-notes.md`** preserves normative FSM research; **`docs/design/mcl-fsm.md`** records the agreed conceptual FSM lifecycles, transitions, output policy and zeroization recovery.
- **Algorithm-specific design and test documents** provide per-algorithm APIs, invariants, scratch memory requirements, test coverage, and evidence; they cite Workbook identifiers rather than duplicating full normative requirements.

Any discrepancy between these documents is an error and should be reported and resolved.

## 4. Logical architecture

```mermaid
flowchart TB
    Caller["Application / boot code / authorized client"]
    MCL["Module Control Layer (MCL)<br/>services, policy, state machine, SSP lifecycle"]
    Crypto["Cryptographic primitives<br/>no_std / no_alloc; caller-provided resources"]
    PIL["Platform Integration Layer (PIL)<br/>memory protection, hardware access, exception / reset entry"]
    HW["MCU hardware / memory / entropy peripheral"]

    Caller <-->|"Five fixed logical-interface mailboxes (MCL mediated)"| MCL
    MCL <-->|"Bounded algorithm calls"| Crypto
    MCL <-->|"Resource control and fault handoff"| PIL
    PIL <-->|"Hardware, interrupt and reset mechanisms"| HW
```

### 4.1 Cryptographic primitives

The cryptographic primitives have these principal responsibilities:

- Conformance to a named algorithm specification, revision, supported parameter set, and defined operation; input validation and deterministic algorithm behavior as applicable.
- Explicit algorithm state and working-storage contracts, including bounded maximum storage and stack requirements for supported configurations.
- Correctness of internal state transitions, error handling, and relevant cleanup of primitive-owned temporary secrets.
- Algorithm-level development tests and evidence, including known-answer and boundary tests, and independent comparisons when available.
- Documentation of side-channel assumptions, implementation variants, and limitations of available assurance.
- An algorithm requiring random inputs declares the exact kind and amount of material needed (for example, deterministic seeds, DRBG-generated bits, or entropy inputs for a DRBG mechanism). Ordinary algorithms do not independently discover hardware sources or fetch entropy; externally supplied inputs are subject to the algorithm's specific contract.


### 4.2 Platform Integration Layer (PIL)

The Platform Integration Layer has these principal responsibilities:

- Binding each supported target to fixed regions for five distinct mailbox payloads and their atomic objects, MCL state, SSPs, plaintext, primitive workspaces and sensitive stacks.
- Enforcing hardware memory protection, privilege/security-domain controls, and transitions independent of primitive cooperation; neither TrustZone nor an MPU is a universal software dependency.
- Ownership of target-specific reset, fault and relevant interrupt-vector entry points, early platform exception handling, hardware fault/reset-cause capture and controlled handoff to the MCL where the processor remains capable of execution.
- Narrowly scoped access to other hardware-dependent services, including entropy-peripheral interfaces and reset-status mechanisms.
- Platform-appropriate mechanisms for MCL-requested zeroization, memory-region retirement, and protection of sensitive locations outside a primitive's direct control.
- The justification, containment, review, and platform testing of necessary `unsafe` Rust operations.


### 4.2.1 Platform-independent PIL protection contract

A selected target's PIL shall enforce the following **architectural
properties**, using hardware mechanisms appropriate to the platform.
These are not prescriptions for a particular MPU or TrustZone register:

1. Fix/bound and separate five mailbox payloads, five atomic synchronization
   objects, MCL/FSM state, SSPs, primitive buffers and sensitive stacks.
2. Deny primitives read/write/execute access to mailboxes and semaphores,
   and unauthorized access to MCL state, SSPs and privilege controls.
3. Enforce least-privilege primitive execution independently of primitive
   cooperation and prevent primitives changing their own access rights.
4. Establish and maintain protections before and throughout sensitive
   execution, accounting for DMA, peripherals and other bus masters.
5. Maintain trusted independent records of **all memory writable by the
   primitive's actual execution context**, not just explicit API buffers.
6. Supply independent atomic mailbox synchronization operations with
   required memory-ordering and visibility guarantees.
7. Handle appropriate fault/reset vectors and hand off trusted context
   to the MCL when processor execution remains possible.
8. Implement MCL-directed zeroization, including the entire
   primitive-writable region set on unexpected internal errors.
9. Reestablish permissions and validate retained mailbox/FSM state
   after reset; refuse affected services if protections cannot be
   established or verified.

The STM32U5A5ZJ's documented MPU, security-attribution, privilege and
exception mechanisms are accepted as engineering assumptions for
continuing design; their operation remains subject to later target,
silicon-revision and errata verification. Unsuitable target mechanisms
do not justify weakening this contract. Reconcile these provisional
architecture requirements with the workbook before design freeze.

### 4.3 Module Control Layer (MCL)

The Module Control Layer has these principal responsibilities:

- Exclusive mediation of the five logical mailbox service interfaces: bounded internal snapshots of untrusted input, validation of those copies, authorization before any outward copy, and atomic publication of completed outbound messages. An adversarial client may disregard semaphore ownership. The MCL is not alone the formal FIPS cryptographic boundary.
- Ownership of module, operation, context and key FSM lifecycles, bounded composition groups, MCL-only working-copy commits and allowable transitions (see [MCL FSM](../design/mcl-fsm.md)).
- Enforcement of the selected security configuration, including output inhibition, SSP lifecycle policy, and required self-test sequencing.
- Coordination with the PIL for bounded protected memory, hardware-source access, and fault/reset handoff; and with primitives for algorithm execution.
- Assessment of retained FSM state following a reset or fault. Unverified or inconsistent retained state shall not be trusted to justify resumption of cryptographic services.

**NOTE:** The actual cryptographic boundary and embodiment remain an explicitly open design matter (Q-001).

## 5. Interface and ownership contracts

The following contracts describe **architectural requirements**, not completed API signatures. Exact types and failure semantics are defined in component design documents and accepted through review.

| Concern | Cryptographic primitives | Platform Integration Layer | Module Control Layer |
| --- | --- | --- | --- |
| Inputs and outputs | Specify authorized internal buffer contracts; never access external mailboxes | Enforce five mailbox payload and semaphore regions separately | Copy bounded untrusted input to internal memory, validate the copy; authorize result before copying outward and publish using atomic signaling |
| Working storage | Declare bounded internal scratch/state needs | Reserve separate mailbox, MCL, SSP and primitive working regions | Allocate internal storage and prevent primitive/mailbox aliasing or conflicting reuse |
| Randomness | Document algorithm-specific seed, pseudorandom-bit or entropy-input requirements; do not access the hardware source implicitly | Provide target-specific entropy-source access and status | Qualify/instantiate the RBG, authorize random inputs, enforce reseed policy and service availability |
| SSP handling | Identify sensitive intermediates and cleanup contract; avoid uncontrolled copies | Protect, isolate and externally zeroize applicable memory regions | Own SSP inventory, usage authorization, lifetime, import/export and module zeroization policy |
| Failures | Distinguish specified negative outcomes from unexpected internal faults; document state and outputs | Own applicable vector/exception entry, capture fault context, provide trusted cleanup mechanisms | Treat unexpected primitive faults as potential active attacks; inhibit services and order zeroization of every region the primitive could modify |
| Tests and evidence | Supply algorithm-level tests, vectors and build/implementation records | Supply target isolation, exception/reset-path, memory cleanup and hardware configuration evidence | Orchestrate applicable module self-tests and integration tests; produce module-level validation evidence |

### 5.1 Mailbox logical boundary

The reference module implements exactly five distinct fixed-memory
payload mailboxes for the ISO/IEC 19790 §7.3.3 logical categories.
**These permissions apply to payloads only** (RO = read only, RW =
read/write, None = no read/write/execute):

| Mailbox | MCL | External | Primitive |
| --- | --- | --- | --- |
| Control input | RO | RW | None |
| Control output | RW | RO | None |
| Status output | RW | RO | None |
| Data input | RO | RW | None |
| Data output | RW | RO | None |

Each mailbox has its **own atomic synchronization object** with separately
defined operation permissions and required acquire/release ordering.
Small control mailboxes will carry message-available, accepted, completed
and related protocol events; actual encodings are not yet selected.

The external client, including the orchestrating host, is **potentially
adversarial**. It may ignore the semaphore protocol and modify input while
the MCL copies it, or read output before the MCL signals completion.
Semaphores establish **message ownership and validity under the interface
contract**; they do not authenticate clients or change the physical
mailbox-access permissions.

The MCL copies a bounded candidate input into protected internal storage
and validates the **copied representation** before dispatch. The copied
bytes need not represent a temporally consistent client-authored message;
the MCL treats them as arbitrary untrusted input and must remain secure
even when the client races the copy. Subsequent changes to the external
mailbox do not affect an accepted operation. Malformed messages cause
defined service rejection rather than a primitive internal-fault response.

For output, the MCL checks completion, authorization, sensitivity and
permissible module state **before copying** authorized results into
externally readable payloads. In particular, plaintext CSPs and other
prohibited information may never be exposed there, including before the
publication signal. A result is **published and valid** only after the
prescribed atomic semaphore transition. Earlier external reads have
undefined *message semantics*: partially written or stale bytes are
not valid results, and no promise of coherent early reads is made.
This does **not** excuse unauthorized disclosure or waive normative
data/control-output inhibition during specified states. In particular,
the target design must still demonstrate that physically readable
outbound mailboxes satisfy ISO/IEC 19790 AS03.07/AS03.10 and applicable
CMVP rules; the exact evidence/mechanism remains open.

The PIL owns actual platform protection and reset/fault entry
mechanisms; the MCL alone mediates logical mailbox information
transfer. Primitives never access mailboxes, and external clients
do not pass pointers into primitive memory.

Physical power, reset/exception signaling and hardware-originating
events are separately handled by the PIL and classified against the
formal cryptographic boundary once Q-001 is resolved. Trusted-channel/
trusted-path implementation is outside the initial reference scope.
Mailbox layouts, message formats and optional checksum/EDC selection
remain open. See [the interface specification](../design/module-interfaces.md).

### 5.2 Operation lifecycle, faults and sensitive-memory cleanup

The module distinguishes **specified service outcomes** from **unexpected internal
errors and processor faults**. Rejection of invalid input, unsuccessful signature
verification and other documented negative results are ordinary service
outcomes, not evidence of an implementation fault. An internal error that should
not occur during correct operation is treated **by policy as a possible active
attack**, irrespective of whether an attack has been demonstrated.

1. **Reset and early entry:** On power-on, processor reset or relevant
   exception entry, the PIL owns the applicable target vectors and early handler.
   Where the processor remains capable of controlled execution, the PIL obtains
   the available reset/fault context, establishes a sufficiently trustworthy
   execution path, and transfers control and fault information to the MCL.
   Cryptographic services and sensitive outputs remain inhibited until the
   MCL has established that operation is permitted. The MCL may inspect retained
   FSM state but shall not rely on it without an appropriate integrity and
   validity determination. If that determination fails, the conservative
   recovery and zeroization policy applies. The distinct rules for cold reset,
   warm reset, watchdog reset and processor faults remain open under Q-008.

2. **Before invocation:** The MCL copies a bounded candidate from the
   untrusted inbound mailbox into internal storage, validates that internal
   representation, authorizes the requested service, checks the relevant FSM
   constraints, and assigns working regions under PIL protection mechanisms.
   Concurrent changes to external input cannot change an accepted operation. The module maintains **trusted
   records independent of the primitive** identifying every memory region
   that the primitive can modify under its actual execution configuration,
   including writable **internal** buffers, working storage and applicable stacks; primitives have no mailbox access.
   Region lifetimes, aliasing and concurrency must satisfy the primitive API
   contract.

3. **During invocation:** The primitive accesses only memory permitted by its
   documented contract. Primitive-created temporary state and possible copies,
   register values, stack usage and workspace sensitivity are documented.
   Writable buffers are not assumed disjoint unless language or API
   guarantees establish this.

4. **On specified completion:** The primitive performs its documented
   cleanup of sensitive temporary state. The MCL then directs the PIL to
   zeroize released sensitive regions. Retained, authorized outputs and
   long-lived SSPs are governed by their explicit lifecycle contracts, rather
   than accidentally erased as scratch. The MCL checks output classification,
   authorization and permissible state before copying any result into an
   external mailbox, then performs atomic message publication. Documented
   negative service outcomes follow the specified completion path.

5. **On unexpected internal error or fault:** The module assumes a possible
   active attack. The MCL inhibits affected cryptographic services and
   commands the PIL to **zeroize every memory region that the primitive was
   capable of modifying**, without narrowing the wipe to locations believed
   to have been altered. The applicable set is determined from trusted
   resource/access records and actual protections, not from metadata supplied
   by the failed primitive. The set includes writable output buffers and
   sensitive regions where the primitive had write capability. Faults that
   bypass normal return reach the PIL through the applicable hardware
   exception/reset path; whenever the MCL can safely execute, the PIL passes
   the fault context and control to the MCL for the security response. The
   module shall not resume affected services before the prescribed
   zeroization, safe reinitialization and applicable tests have succeeded.

6. **When normal recovery cannot execute:** A fault may prevent the MCL or
   even the PIL from executing. The available platform mechanisms must
   prevent affected state from being used or exposed through subsequent
   services until the required zeroization has been established. The target
   design must analyze retained SRAM, reset-cause evidence, persistent FSM
   metadata, early-boot access controls, interrupted cleanup and watchdog
   paths. This architecture does not claim that software can erase memory
   while the processor cannot execute or power is unavailable. Trusted
   fallback behavior and the conditions for safe recovery are to be
   specified and tested under Q-008.

7. **Self-test requests:** Pre-operational tests, conditional algorithm
   self-tests and operator-initiated/on-demand tests have separately specified
   state-transition and output-inhibition requirements. Invoking a test does
   not by itself imply indiscriminate erasure of all live SSPs. Any fault
   detected by a test invokes the applicable module error policy. Detailed
   sequencing belongs to the future MCL FSM specification.

For sensitive memory, a source-level assignment to zero is not evidence that
compiler-optimized machine code erased every copy. Implementation-specific
wipes, generated-code review, target tests, exception paths, registers, hardware
buffers, DMA activity and stack copies require their own evidence. Neither safe
Rust nor static allocation alone establishes complete erasure.

**Memory-cleanup division:** Portable primitives implement and document
local sensitive-temporary cleanup; the PIL implements externally controlled
platform-dependent zeroization, including fault/reset execution paths; the MCL
decides and coordinates SSP lifecycle, module error response and the complete
primitive-writable-region wipe required by an unexpected fault. This preserves
D-008–D-010 and the scope of SSP-007/008/012 while leaving platform recovery
details open under Q-008.

### 5.2.1 Agreed finite-state and recovery architecture

The detailed design baseline is [the MCL FSM](../design/mcl-fsm.md).
The module FSM gates startup, required self-tests, service/role authorization,
zeroization and error handling. Operation acceptance occurs only after a
bounded internal snapshot is validated and all required resources acquired.
The operation FSM separates preparation, primitive execution, evaluation,
MCL-only commit, output authorization/copy and semaphore publication.

All authoritative contexts and keys are inaccessible to primitives,
which execute exclusively on protected temporary working copies.
Successful primitive completion authorizes the MCL to commit state
under the defined service contract. A failed/canceled accepted
stateful operation retires participating mutable contexts and
dependent fixed-group members; an ordinary GCM tag failure **does not**
invalidate a trustworthy AES key. `RETIRING` is a transient one-way
context state until safe cleanup returns the slot to `FREE`.
All volatile contexts are discarded after reset.

Both deterministic and RBG-based AES-GCM IV strategies are supported
as key-bound policies. MCL allocation irrevocably spends IVs/usage
budget **before** encryption; no cancellation or reset recycles them.
Loss of IV assurance permanently changes a still-trustworthy,
decryption-authorized key to `DECRYPT_ONLY`; compromised key integrity
requires invalidation instead. Large single-message authenticated
decryption is **bounded** by protected RAM: no plaintext is exposed
before final GCM tag verification, requests exceeding the configured
limit are rejected before acceptance, and no module-defined
streaming-record protocol is included. The limit is deferred.

Unexpected primitive faults trigger zeroization of **every actual
primitive-writable region** using independent trusted PIL records.
Module error does not automatically require all-key zeroization;
scope and mandatory triggers are specified separately. Retained SSP
region erasure uses a durable in-progress marker cleared only after
completed zeroization, so interrupted cleanup is retried after reset.
Storage-specific encoding and rollback protection remain open (Q-008).

Routine scratch clearing and per-context `RETIRING` cleanup do **not**
force the global FSM into `ZEROIZING`. Instead, a protected
data-output inhibition guard covers actual SSP erasure even while
the module stays operational; the authorized zeroization service
and applicable whole-module purge remain separately modeled.

Protocol-bound MACsec GCM-AES and MACsec GCM-AES-XPN encryption
profiles must be accommodated as **distinct deterministic IV
constructions**, bound to protected key/transmit-association
identity, SCI or SSCI/salt, full-width 32- or 64-bit PN and
irreversible per-association allocation/rollback state. A caller's
PN proposal is never unrestricted authority to select encryption
IVs. MACsec decryption may consume externally derived IVs.
A future approved MACsec service must demonstrate FIPS 140-3
IG §C.H Scenario 1(c)'s distributed validated-module responsibilities,
Scenario 3a power-restoration rules, and Additional Comment 8's
prohibition on static pre-shared SAKs; it does **not** require us
to implement the MACsec stack. See
[the MCL FSM MACsec profile](../design/mcl-fsm.md#51-macsec-and-macsec-xpn-protocol-bound-iv-profiles).

These are architectural decisions, not target verification or completed
CMVP evidence. The binary workbook requires later reconciliation.

### 5.3 Trust, privilege and safety boundaries

Distinguish at least four concepts:

- **Algorithm/component API boundary:** A software contract between caller and implementation.
- **Memory and privilege boundary:** Enforced by the target hardware, runtime configuration and trusted code; its extent may differ from Rust crate boundaries.
- **Module cryptographic boundary:** Established for a specific FIPS 140-3 submission, including the implementation and relevant operational environment.
- **Physical boundary/perimeter:** Depends on the actual device embodiment and applicable validation requirements.

None is implied by another; in particular, the MCL's control of the external
service interfaces does not itself define the complete module cryptographic
boundary. 'Secure world' and 'privileged CPU mode' are distinct attributes on
applicable Arm microcontrollers. The current engineering preference is small privileged configuration/ISR code with bulk cryptography unprivileged where feasible in the secure domain (D-007); exact trusted-code inclusion, hardware mappings, and evaluation remain Q-001 and related target questions.

### 5.4 Secret transfers and output policies

The MCL may receive plaintext secrets through the untrusted data-input mailbox, subject to service and SSP-entry authorization and validation of an internal copy. However, it shall enforce the adopted policy against plaintext external export of CSPs and shall classify every external path. Public keys, signatures and ciphertexts may have different disclosure and integrity requirements. The specific ML-KEM shared-secret consumer/handle/export contract is **not yet frozen** (Q-005). Encrypted SSP backup does not protect confidentiality against a compromised host that chooses or knows the password from which the backup-wrapping key is derived; the design shall document that threat-model limitation. A separately credentialed organizational recovery authority and ML-KEM-based wrapping scheme are outside the present reference-module scope. The architecture does not silently authorize a raw shared-secret module service.

## 6. Random-bit generation, state, and services

Cryptographic primitives shall make any seed, entropy or pseudorandom-input dependency explicit and algorithm-specific. A portable **DRBG mechanism** may perform the deterministic algorithm and accept explicitly supplied instantiation/reseed inputs, while **entropy qualification, hardware-source integration, approved-construction claims, instantiation and reseed policy, service dispatch and failure response** belong to the MCL and reference-module integration, using platform mechanisms provided by the PIL.

The current Workbook contains selected design directions for hardware entropy, SHA-256 Hash_DRBG, output allowance, and asynchronous prefetch (D-002, D-003, D-006). This document does **not** restate their parameter values as independent requirements. The formal RBG claim, tested device match, demand bounds, concurrency, and recovery remain open (Q-002, Q-003, Q-006, Q-008).

Likewise, password-based backup (D-011/D-012), key import/restore, authorization, and policy-level SSP output restrictions belong to the module service design, though they consume primitives. Neither a general-purpose primitive nor the PIL independently provides a CMVP-approved backup construction.

## 7. Deployment and implementation variants

The initial architectural preference is **one portable, safe Rust implementation per supported primitive**, with identified feature selections and no mandatory `std` or `alloc` runtime dependency. Distinct assembly, SIMD, accelerator, or configuration-dependent implementations are separate variants requiring explicit selection and additional evidence.

A planned embedded reference target is the **STM32U5A5ZJ** on the **NUCLEO-U5A5ZJ-Q** board (D-001). This is a development direction, **not** an accepted determination of FIPS embodiment, operational environment, physical security applicability, or entropy-source certificate reuse (Q-001, Q-003). Hosted macOS and GCP/Linux development systems are not thereby included among validated operational environments.

Generic bare-metal compilation in CI checks portability only for that tested target and configuration. It does not demonstrate correct execution, resource bounds, memory isolation, or CMVP readiness on the intended MCU.

## 8. Assurance boundaries and evidence

Evidence is developed at the level where its claim can be established:

| Evidence area | Primary owner | Integration obligation |
| --- | --- | --- |
| Algorithm conformance, API behavior and negative cases | Primitive | Identify the exact implementation/parameters, perform applicable independent testing and satisfy any additional module obligations |
| Stack/scratch bounds, error handling, local cleanup | Primitive | Confirm behavior under the selected compiler, build flags and target environment |
| Memory maps, five mailbox/semaphore regions, access controls, DMA/debug access, fault/reset vectors and hardware source conditions | PIL | Verify isolation, bounded input snapshot and validation, prohibition on sensitive early disclosure, required output inhibition, retained-state handling and boundary inclusion |
| Approved services/modes, SSP lifecycle, full primitive-writable-region fault zeroization, module error state and output inhibition | MCL with PIL | Specify, implement, test and document the complete module obligations and fault/reset execution paths |
| Algorithm validation protocols/certificates, CAST/PCT integration and CMVP submission | Module/integrator with laboratory as appropriate | Establish exact validated implementation, operational environment and required program evidence |

Successful project tests, a `no_std` build, or reuse of an entropy-source validation do not confer FIPS 140-3 module validation. Algorithm validation and module validation are distinct. The actual claimed implementation, compiled artifact, environment, and applicable CMVP requirements must be identified and assessed separately.

The Workbook's evidence IDs (E-001–E-027) remain the master evidence planning references. Evidence ownership may be cross-layer, and completion of a library artifact is not completion of its corresponding module-level test obligation. Where a requirement combines distinct obligations, preserve its ID and create separately traceable implementation/test records rather than silently marking the entire requirement complete.
