# Module FSM — Normative Requirements Notes

**Status:** Normative research retained; conceptual MCL FSM architecture now documented separately  
**Date:** 2026-10-20

This note retains the normative survey separately from the agreed
[conceptual MCL FSM design](../design/mcl-fsm.md). Interface and PIL
mechanism details still require implementation and testing.
The workbook remains the authority for CMVP planning and applicability.

## Prescribed minimum states and representation

ISO/IEC 19790:2012 §7.11.4, AS11.08–AS11.13, requires a finite-state
model or equivalent represented by **state descriptions, a transition table
and a diagram**, detailed enough to demonstrate compliance.

AS11.10 specifies power on/off, general initialization, Crypto Officer,
CSP entry, approved, self-test, and error states, plus User state if the
User role is implemented. Bypass and quiescent are examples of optional
states. AS11.11 requires recovery from non-hard errors; hard errors requiring
service/repair are excepted. AS11.12 calls for separate depiction of each
distinct service, security-function use, error state, self-test, or operator
authentication. AS11.13 prohibits transitioning into Crypto Officer state
directly from another role; role-clearing/re-authorization requirements must
be addressed if role switching is supported.

ISO/IEC 24759:2017 §6.11.4 VE/TE11.08 requires complete and mutually
consistent states, transitions, descriptions, data/control inputs and
observable outputs. Among other matters, the tester assesses disjoint
states, unambiguous input conditions, paths from power-on to reachable
states and from non-power-off states to power-off, and defined actions for
all possible combinations of input and control events. TE11.13 addresses
the Crypto Officer role restriction.

## Cross-cutting transition and output constraints

- ISO/IEC 19790 §7.3.3 AS03.07 prohibits data output during manual SSP
  entry, pre-operational testing, firmware loading, zeroization and error
  state. AS03.10 restricts control output during error states, with
  documented exceptions. ISO/IEC 24759 tests also check control-output
  inhibition during self-tests.
- §7.4.3 AS04.12–AS04.17 mandates module identification/version, show
  status, initiate tests, approved security functions and zeroization.
- §7.10 AS10.01–AS10.03 and AS10.15–AS10.20 governs internal test
  determination, pre-operational tests and sequencing.
- AS10.07–AS10.11 requires error-state entry, error indication, and
  prevention of prohibited operations after failed self-tests.
- AS10.25–AS10.35 covers applicable conditional algorithm tests and
  pairwise consistency tests, which may gate the first operational use.
- AS10.53 provides operator on-demand test initiation at Level 1. AS10.54
  automatic periodic tests apply to higher levels.
- §7.2.4.3 AS02.25–AS02.32 constrains optional degraded operation; it
  is not selected for the initial reference design.

NIST SP 800-140 and the current FIPS 140-3 Implementation Guidance may
modify or clarify the applicable requirements and must be checked at the
design/validation gates. DRBG and entropy mechanisms introduce additional
SP 800-90A/B/C lifecycle, health-test, and failure constraints.

## Specific CMVP guidance examined 2026-10-23

- **Routine zeroization and the module FSM.** Internal cleanup of
  temporary sensitive storage while `RELEASING` or `RETIRING`
  need not enter the *global* `ZEROIZING` state, although
  ISO/IEC 19790 AS03.07's **data-output inhibition while
  zeroizing SSPs** still applies. Output inhibition is an
  independent MCL/PIL guard. IG §9.7.B (2026-08-19,
  applicable to Levels 2–4) permits implicit completion
  indicators via the function/service that zeroized
  temporary SSPs. The Level 1 project should not
  mistake that indicator provision for a rule imposing a
  global state transition on every wipe. The operator
  zeroization service remains a distinct state/behavior.
- **MACsec and XPN.** Current IG §C.H Scenario 1(c)
  explicitly recognizes IEEE 802.1AE GCM-AES and
  GCM-AES-XPN cipher suites, with distributed roles
  across separately validated MACsec components.
  Scenario 3a governs recovery of deterministic IV state
  after power loss. Additional Comment 8 prohibits
  statically configured pre-shared SAKs in approved mode.
  Correct key+association SCI/SSCI/salt binding and full
  PN monotonicity are important local controls, not a
  substitute for the distributed protocol conditions.
  Receiving/decrypting externally derived IVs is distinct
  from encrypting with an externally proposed PN.
- The applicable source is the [2026-08-19 FIPS 140-3
  IG](https://csrc.nist.gov/csrc/media/Projects/cryptographic-module-validation-program/documents/fips%20140-3/FIPS%20140-3%20IG.pdf)
  §§9.7.B and C.H. Map these findings into the workbook
  before making an implementation or compliance claim.

## Project policy, not an ISO-imposed full-region wipe

Unexpected primitive internal errors are treated as potential active
attacks. The MCL inhibits affected services and the PIL zeroizes **every
region that the primitive execution context could modify**, not only
regions believed to have changed. The PIL owns the applicable fault/reset
entry mechanism; the MCL verifies any retained state before permitting
cryptographic operation. This full-region fault wipe is project security
policy, separate from the standard's specific zeroization obligations.

## Design handoff

The MCL FSM must eventually account for the five mailbox interfaces,
input acceptance *after internal-copy validation*, outbound release
authorization *before mailbox copying*, atomic message publication,
synchronizing objects, resets, invalid transactions, normative output
inhibition, error and approved-service indications. Early external reads
are not valid published messages; an adversarial client may ignore
semaphores, and this does not relax SSP protection or output inhibition. No precise semaphore transitions or control-message
encodings are adopted yet; see
[module-interfaces.md](../design/module-interfaces.md).

A conceptual hierarchical design, state descriptions, major event guards,
context and key lifecycles, fault scopes, output-copy authorization and
interrupted-zeroization policy now appear in
[mcl-fsm.md](../design/mcl-fsm.md). The **exhaustive** transition and
observable-output table, role/service mappings, current CMVP guidance
traceability, target evidence and implementation remain unfinished.
This note is a normative-research checkpoint, not compliance evidence.
