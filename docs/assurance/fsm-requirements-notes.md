# Module FSM — Normative Requirements Notes

**Status:** Requirements research retained; no MCL FSM has been designed  
**Date:** 2026-10-20

This note preserves the prior finite-state-model requirements survey while
the logical service interface and PIL mechanism contracts are specified.
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
input acceptance, output publication, synchronizing objects, resets,
invalid transactions, output inhibition, error and approved-service
indications. No precise semaphore transitions or control-message
encodings are adopted yet; see
[module-interfaces.md](../design/module-interfaces.md).

The actual state machine and traceability tests are intentionally **not
yet designed**. This note is a durable research checkpoint, not evidence
of implementation or validation.
