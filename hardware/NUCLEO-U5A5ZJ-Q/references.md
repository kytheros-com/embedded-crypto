# STM32U5A5 technical reference catalogue

**Status:** Engineering reference index; subject to human review

**Target:** NUCLEO-U5A5ZJ-Q, STM32U5A5ZJ revision X
**Catalogue date:** 2026-11-26

## Purpose and local document management

This catalogue gives the developer, Codex, ChatGPT, and future reviewers
stable names for the local technical references used in Platform
Integration Layer (PIL) design and hardware characterization. It indexes
sources; it neither adds PIL requirements nor changes the MCL design or
assurance baseline. The [architecture](../../docs/assurance/architecture.md)
§4.2 and [module interfaces](../../docs/design/module-interfaces.md) §1
assign platform mechanisms to the PIL. The
[documentation standard](../../docs/documentation-standard.md) §2–3
requires traceable claims and explicit unresolved discrepancies.

The 16 originals live in Git-ignored `Manuals/` for local use. Git
`check-ignore` confirms the directory and every supplied file are ignored;
`git ls-files --stage` and the staged diff show none tracked or staged.
The files are copyrighted and must not be copied into tracked content.
Their SHA-256 digests identify exact local bytes, not publisher
authenticity, correctness, or redistribution rights. The catalogue
contains metadata, navigation cues, and short functional descriptions,
not extracted pages or decision tables. Temporary analysis used local
text and Open XML extraction outside the tracked repository.

The documents themselves identify the publisher sites
[Arm](https://www.arm.com/) and [STMicroelectronics](https://www.st.com/).
These are **publisher URLs**, not verified direct download URLs for each
file. Local accessibility is independent of online availability.

## Reference hierarchy and interpretation

| Scope | Stable IDs | Use |
|---|---|---|
| Architectural specification | ARM-ARM, DDI0624, DDI0625 | Rules, permitted outcomes, instruction and exception semantics. |
| Processor implementation | ARM-TRM, ARM-ERRATA | Cortex-M33 implementation and known processor deviations. |
| MCU implementation and integration | RM0456, DS13543, ES0553 | ST integration, device properties, and silicon limitations. |
| Development board | UM2861 | Physical board configuration and connections. |
| Programming and security guidance | ARM-DGUI, ARM-EXCEPTIONS, ARM-SECURE-GUIDE, ARM-STACK-SEAL, PM0264, AN4838, AN5347 | Explanations and engineering practices. |

This is a scope map, not a simple precedence ladder. Applicable errata
can qualify documented behavior; the processor TRM supplies properties
where the architecture permits variation; ST manuals describe MCU
integration; security guidance may recommend restrictions beyond
architectural rules. A board observation does not establish an
architectural guarantee. Record and investigate disagreement rather
than choosing a convenient interpretation.

## Target applicability

The established board baseline is NUCLEO-U5A5ZJ-Q with STM32U5A5ZJ
revision X, an Arm Cortex-M33 implementing Armv8.0-M with Main
Extension. The expected processor revision is r0p4, supported by the
local r0p4 processor guides and the ES0553 core reference but **not yet
independently confirmed by CPUID on this board**. The Security
Extension is implemented by the platform but the current experiment
configuration disables it (`TZEN=0`). Processor support, option-byte
state, and firmware activation are different facts. Privilege, PSP,
MPU, and UsageFault have been exercised in the linked
[hardware ledger](test-ledger.md); that evidence is limited to its
recorded configurations.

ARM-ARM DDI0553B.z integrates Armv8.0-M and Armv8.1-M material and
optional extensions. A feature appearing there does not establish its
presence on this Cortex-M33 or its activation in firmware. CPUID,
feature registers, silicon revision, option bytes, and configuration
must be checked for each implementation-dependent claim. DMA and
other bus-master isolation, GTZC behavior, TrustZone-enabled routing,
retention, and recovery are not established by the current `TZEN=0`
UsageFault experiments.

## Reference inventory and provenance

Stable IDs survive revision changes; the revision in each row identifies
**this local file**. For Arm, the processor revision column is a
statement of the document's scope, not independent CPUID evidence.
Dates are document release/issue dates printed by the publisher unless
qualified below. `—` means the document has no processor-revision
scope; `unresolved` means the supplied file does not unambiguously
state the requested metadata.

| ID | Publisher; exact title | Document number; local revision/issue | Release date; processor revision |
|---|---|---|---|
| AN4838 | STMicroelectronics; *Introduction to memory protection unit management on STM32 MCUs* | AN4838 Rev. 11 | Dec. 2025; multi-core guidance |
| AN5347 | STMicroelectronics; *Introduction to Arm® TrustZone® features on STM32L5, STM32U5, and STM32U3 MCUs* | AN5347 Rev. 7 | Feb. 2025; Cortex-M33 families |
| DS13543 | STMicroelectronics; *STM32U5Axxx — Ultra-low-power Arm® Cortex®-M33 32-bit MCU+TrustZone®+FPU, 240 DMIPS, 4 MB Flash, 2.5 MB SRAM, LTDC, MIPI® DSI, crypto* | DS13543 Rev. 3 | May 2025; integrated Cortex-M33 |
| ES0553 | STMicroelectronics; *STM32U59xxx and STM32U5Axxx device errata* | ES0553 Rev. 6 | June 2026 in PDF; core reference r0p4 |
| PM0264 | STMicroelectronics; *STM32 Cortex®-M33 MCUs and MPUs programming manual* | PM0264 Rev. 8 | Mar. 2026; Cortex-M33 generic |
| RM0456 | STMicroelectronics; *STM32U5 series Arm®-based 32-bit MCUs* | RM0456 Rev. 7 | Mar. 2026; integrated Cortex-M33 |
| UM2861 | STMicroelectronics; *STM32U5 Nucleo-144 board (MB1549)* | UM2861 Rev. 10 | Mar. 2025; board guide |
| ARM-DGUI | Arm; *Arm® Cortex®-M33 Devices Generic User Guide* | 100235_0004_00_en, issue 0004-00 | 10 Apr. 2018; r0p4 |
| ARM-STACK-SEAL | Arm; *Armv8-M Architecture — Stack sealing and why it is needed in TrustZone for Armv8-M* | 102446, Issue 1.0 | 28 Apr. 2021; Armv8-M |
| ARM-EXCEPTIONS | Arm; *Armv8-M Exception Model User Guide* | 107706_0100_01_en, Version 1.0, Issue 01 | 19 Jul. 2023; Armv8-M |
| ARM-ERRATA | Arm; *Cortex-M33 AT623 and Cortex-M33 with FPU AT624 Software Developer Errata Notice* | SDEN-756493 v14.0 | 8 Mar. 2022; revision-specific entries through r1p0 |
| ARM-TRM | Arm; *Arm® Cortex®-M33 Processor Technical Reference Manual* | 100230_0004_00_en, issue 0004-00 | 18 Apr. 2018; r0p4 |
| ARM-ARM | Arm; *Armv8-M Architecture Reference Manual* | DDI0553B.z, ID06022026 | 6 Feb. 2026; v8.0-M plus integrated v8.1-M material |
| ARM-SECURE-GUIDE | Arm; *Secure software guidelines for Armv8-M* | 100720_0300_00, Issue 0300 | 30 Jul. 2020; Armv8-M guidance |
| DDI0624 | Arm; *Armv8-M Faults on Instruction Fetch* | ARM DDI 0624; explicit document revision unresolved | Cover: 8 Mar. 2024; revision-history row: 29 Jul. 2024; Armv8-M |
| DDI0625 | Arm; *Armv8-M Faults on Exception Handling* | ARM DDI 0625; explicit document revision unresolved | Cover: 8 Mar. 2024; revision-history row: 29 Jul. 2024; Armv8-M |

The following table records original files in `Manuals/`. PDF page
counts are physical one-based pages; XLSX has no PDF page count. The
filenames are exact and relative to that directory. All entries were
locally accessible without altering the originals.

| ID | Local filename | Format | Bytes | PDF pages |
|---|---|---:|---:|---:|
| AN4838 | `an4838-introduction-to-memory-protection-unit-management-on-stm32-mcus-stmicroelectronics.pdf` | PDF | 651,865 | 24 |
| AN5347 | `an5347-introduction-to-arm-trustzone-features-on-stm32l5-stm32u5-and-stm32u3-mcus-stmicroelectronics.pdf` | PDF | 1,912,583 | 40 |
| DS13543 | `DS_stm32u5a5zj.pdf` | PDF | 4,822,728 | 389 |
| ES0553 | `es0553-stm32u59xxx-and-stm32u5axxx-device-errata-stmicroelectronics.pdf` | PDF | 1,647,110 | 38 |
| PM0264 | `pm0264-stm32-cortexm33-mcus-programming-manual-stmicroelectronics.pdf` | PDF | 11,499,369 | 239 |
| RM0456 | `rm0456-stm32u5-series-armbased-32bit-mcus-stmicroelectronics.pdf` | PDF | 73,588,589 | 3,651 |
| UM2861 | `um2861-stm32u5-nucleo144-boards-mb1549-stmicroelectronics.pdf` | PDF | 3,912,163 | 52 |
| ARM-DGUI | `arm_cortex_m33_dgug_100235_0004_00_en.pdf` | PDF | 1,758,530 | 342 |
| ARM-STACK-SEAL | `Armv8-M Architecture-Stack sealing and why it is needed in TrustZone for Armv8-M.pdf` | PDF | 613,291 | 20 |
| ARM-EXCEPTIONS | `armv8-m_exception_model_user_guide_107706_0100_01_en.pdf` | PDF | 2,245,138 | 120 |
| ARM-ERRATA | `Cortex_M33_AT623_and_Cortex_M33_with_FPU_AT624_Software_Developer_Errata_Notice_v14_0.pdf` | PDF | 456,099 | 60 |
| ARM-TRM | `cortex_m33_trm_100230_0004_00_en.pdf` | PDF | 980,164 | 162 |
| ARM-ARM | `DDI0553B_z_armv8m_arm.pdf` | PDF | 16,439,994 | 2,149 |
| ARM-SECURE-GUIDE | `Secure_software_guidelines_for_Armv8.pdf` | PDF | 515,792 | 27 |
| DDI0624 | `DDI0624_armv8M_faults_on_instruction_fetch.xlsx` | XLSX | 44,053 | — |
| DDI0625 | `DDI0625_armv8M_faults_on_exception_handling.xlsx` | XLSX | 65,260 | — |

SHA-256 digests of the original bytes:

| ID | SHA-256 |
|---|---|
| AN4838 | `21a3b82d435f5501e10e2663065c987d30f485f13ce3f26f51c59b1e9a90b002` |
| AN5347 | `f37f14c44d854ca91595d1874177b82f91474e9fb84467722f9bb63fa4bfc99a` |
| DS13543 | `62f035250fb393509554739e3b4e23d49321a4fcfbfb39fd3d33e99a3ac23760` |
| ES0553 | `57670b5a1bb398388515909b0a22f54a08c17923d02dbf11206e39bc89e5ee26` |
| PM0264 | `6e42f26331c28aab3ef88d4a271c9e596b10090a594ebc2011edd1a265e50903` |
| RM0456 | `6edaaaae0c5d57c58ffc3a576d44e7b62e8d9cd0e495e8f61bab688226fc327c` |
| UM2861 | `2ad8e32398e92f2a2d803c7c436bbe85bd15d4c187df5d3fc57840159da977e2` |
| ARM-DGUI | `fda45f474c0272d690d89371e207d0348c947a0a33a7c10f9698bab5bbe0cbee` |
| ARM-STACK-SEAL | `556a17885d14146a6d4e3a89eb07e898a2786803da2c3f89d60fe7a8f00ccd2f` |
| ARM-EXCEPTIONS | `a2743dab593346bb2a4178f5f1bda1dee86451cc5142a1c187041cf8d9d67f3e` |
| ARM-ERRATA | `f222c1f927a6e2c053193419b0b5f65d9b894eefdb81f7104c31bbec02496dde` |
| ARM-TRM | `16f7be5a1ad6e6c44aafe48bb7445e3fa31a5e08b51f47d76de55ca9a1d36bab` |
| ARM-ARM | `468594b22cf5a4bae1261f2cd39d5aa51040fbb566be18cc358c169918fd7a72` |
| ARM-SECURE-GUIDE | `53cf1a063f407208de05a7dadf643ace4bbc776e07112718aa22e2a89b5da5a6` |
| DDI0624 | `6d835b13c01053ac6360486898a1a0a5c822392de93b92525a4e1103ca618937` |
| DDI0625 | `b1c57806577df2ceefb321e4790f09843dfe8795aa3d7f5a9d8fa5877cc77b71` |

### Applicability and primary engineering use

| ID | Applicability and primary use |
|---|---|
| ARM-ARM | Applicable Armv8.0-M rules; locate normative semantics and permitted variation, filtering out unimplemented optional/v8.1-M features. |
| DDI0624 | Supplemental architectural instruction-fetch outcome cases; apply only matching feature and security conditions. |
| DDI0625 | Supplemental architectural exception-entry/return, tail-chain and FP preservation cases; apply only matching conditions. |
| ARM-TRM | r0p4 Cortex-M33 implementation options, feature registers, SAU/MPU/NVIC/debug properties. |
| ARM-ERRATA | Processor erratum IDs and revision applicability; check relevant r0p4/AT623 or AT624 entry and configuration before using any workaround. |
| RM0456 | STM32U5A5 memory, GTZC, DMA, reset, RAMCFG, peripherals and debug integration. |
| DS13543 | STM32U5A5ZJ device features, pin/package, memory size and electrical limits. |
| ES0553 | STM32U5A5ZJ revision-X device limitations and documentation errata. |
| UM2861 | NUCLEO-U5A5ZJ-Q MB1549 board, power, LED, reset, debug and connector configuration. |
| ARM-DGUI | r0p4 programming model and core-peripheral explanation; guidance, not replacement for ARM-ARM/TRM. |
| ARM-EXCEPTIONS | Exception/fault teaching guide and examples; check normative details against ARM-ARM. |
| ARM-SECURE-GUIDE | Security-state entry, validation, data-leakage and reentrancy guidance for future TZEN-enabled work. |
| ARM-STACK-SEAL | Secure-stack-seal rationale and failure examples for future TZEN-enabled work. |
| PM0264 | ST Cortex-M33 programming model, fault registers and exception guidance relevant to this MCU family. |
| AN4838 | STM32 Armv8-M MPU setup examples; explanatory, not a substitute for target register descriptions. |
| AN5347 | STM32U5 TrustZone and GTZC overview; future TZEN-enabled integration planning only. |

## Engineering subject index

The entries point to verified section/chapter headings, not to a review
of every page. A reference's scope and optional-feature conditions
still need checking before applying a rule. PDF page citations in later
work must be verified at the point of use. The DDI workbook coordinates
are given in the spreadsheet section below.

| Subject | Starting points |
|---|---|
| Processor privilege and execution modes | ARM-ARM §B3.2; ARM-DGUI §2.1; PM0264 §2.1. |
| CONTROL register semantics | ARM-ARM §§B3.4, D1.2.13; PM0264 §2.1.3.8. |
| MSP, PSP, and stack-limit registers | ARM-ARM §§B3.8, B3.21, D1.2.177, D1.2.218; ARM-DGUI §2.1. |
| Exception entry and return | ARM-ARM §§B3.18–B3.20, B3.22; ARM-EXCEPTIONS §2.2; PM0264 §2.4.7. |
| EXC_RETURN | ARM-ARM §§B3.22, D1.2.96; ARM-EXCEPTIONS §2.7; PM0264 §2.4.7.2. |
| Exception-frame layout and validity | ARM-ARM §§B3.19, B3.21, B3.24–B3.25; ARM-EXCEPTIONS §2.6; DDI0625 relevant worksheet. |
| UsageFault classification | ARM-ARM §B3.12; ARM-EXCEPTIONS §3.1; PM0264 §4.3.12.3. |
| MemManage fault classification | ARM-ARM §B3.12; ARM-EXCEPTIONS §3.1; PM0264 §4.3.12.1. |
| BusFault classification | ARM-ARM §B3.12; ARM-EXCEPTIONS §3.1; PM0264 §4.3.12.2. |
| SecureFault classification | ARM-ARM §B3.12; ARM-EXCEPTIONS §3.1; ARM-SECURE-GUIDE §3.4. |
| Fault-address validity and retention | ARM-ARM §§D1.2.6, D1.2.166–167; PM0264 §§4.3.12.1–4.3.12.2. |
| Fault precedence and simultaneous faults | ARM-ARM §B3.12; DDI0624 `V8MFETCH`; DDI0625 `Exc entry`; ARM-ERRATA summary/entry. |
| HardFault escalation | ARM-ARM §B3.31; ARM-EXCEPTIONS §3.5; PM0264 §4.3.13. |
| Faults during stacking and unstacking | ARM-ARM §§B3.24–B3.25; DDI0625 `Exc entry`, `Exc return post tail-chain`. |
| Nested exceptions | ARM-ARM §§B3.24–B3.25; ARM-EXCEPTIONS §§2.2, 2.5. |
| Tail-chaining and late arrival | ARM-EXCEPTIONS §2.12; ARM-ARM §§B3.19, B3.24; DDI0625 tail-chain worksheet. |
| Processor lockup | ARM-ARM §B3.33; ARM-ERRATA relevant entry. |
| NVIC priority and interrupt routing | ARM-ARM §§B3.9–B3.11, B3.13, B12; ARM-TRM §B4; RM0456 §3.5. |
| Secure interrupt deprivileging | ARM-EXCEPTIONS §5.7; ARM-SECURE-GUIDE §3.8; ARM-ARM §B3.13. |
| MPU configuration and enforcement | ARM-ARM §B10.1; ARM-TRM §B3.3; RM0456 §§3.6.3, 3.7.1; AN4838 §§4, 7. |
| SAU and IDAU configuration | ARM-ARM §§B10.3–B10.4; ARM-TRM §B3.2; RM0456 §3.5.3; AN5347 §3.3. |
| GTZC configuration and enforcement | RM0456 §§3.5.4, 5.3–5.11; AN5347 §4.2. |
| DMA and other bus-master isolation | RM0456 §§3.5.4, 17.4.14–17.4.15, 18.4.14–18.4.15. |
| Security-state transitions | ARM-ARM §B3.15; ARM-DGUI §2.4; ARM-SECURE-GUIDE §§1.1–1.2; AN5347 §3. |
| Secure gateway veneers | ARM-SECURE-GUIDE §1.3; ARM-ARM §B3.15. |
| Secure stack sealing | ARM-STACK-SEAL §§2–5; ARM-SECURE-GUIDE §3.5. |
| Register sanitization | ARM-ARM §B3.20; ARM-SECURE-GUIDE §2.3. |
| Secure reentrancy | ARM-SECURE-GUIDE §3.7; ARM-STACK-SEAL §5. |
| Non-secure memory volatility | ARM-SECURE-GUIDE §3.2. |
| Memory barriers and synchronization | ARM-ARM §§B7.2.9–B7.2.16; PM0264 §2.1.3.8 for ISB after CONTROL.SPSEL. |
| Exclusive monitors and semaphores | ARM-ARM §§B9.1–B9.5; ARM-TRM §B1.5. |
| Reset mechanisms and memory retention | ARM-ARM §B1; RM0456 §§2.3.3, 11.3, 64.4; UM2861 §7.8. |
| Watchdog mechanisms | RM0456 §§61–62; UM2861 §7.8 for board reset context. |
| SRAM ECC | RM0456 §6.3.2; ES0553 system errata summary and applicable entry. |
| Hardware-assisted SRAM erasure | RM0456 §§6.3.5, 6.6.10; verify exact region coverage before use. |
| Tamper detection | RM0456 §§3.7.3, 64.4.9–64.4.11; AN5347 §8. |
| Environmental fault protection mechanisms | RM0456 §§10.6.1–10.6.4, 11.4.11–11.4.12; DS13543 electrical limits. |
| Debug access and authentication | ARM-ARM §B13.3; ARM-TRM §C1; RM0456 §§3.11, 75.2.6, 75.12; UM2861 §7.3. |
| Applicable processor and silicon errata | ARM-ERRATA summary/numbered entry with revision column; ES0553 §§1–2, Table 2 for X. |

## Standard citation convention

Cite the stable ID **and the revision actually consulted**. For PDFs,
add the section or chapter and printed page number when available,
plus an architectural rule identifier when the source supplies one.
Examples (the cited coordinates must be checked before reuse):

- `[ARM-ARM, DDI0553B.z, §B3.4, printed p. 73, RSNGJ]`
- `[RM0456, Rev. 7, §6.3.5, printed p. 280]`
- `[ARM-SECURE-GUIDE, Issue 0300, §3.4, printed p. 22]`
- `[ARM-ERRATA, SDEN-756493 v14.0, erratum 2219175]`

Use **printed** page numbers for normal citations; if a PDF viewer's
physical page differs, state both, e.g. `printed p. X (PDF page Y)`.
PDF pages are one-based. The ARM-ARM includes front matter and
architecture section labels; do not assume its PDF page index equals
its printed page. For architectural pseudocode, give the exact
function or procedure name and its rule identifier, if supplied, and
label it as pseudocode rather than explanatory prose. For registers,
include the register, field, section, and revision, e.g.
`[ARM-ARM, DDI0553B.z, CONTROL.SPSEL, §D1.2.13]`.

For spreadsheets, cite the verified workbook revision if one exists;
for these files, use `revision not stated, SHA-256 <digest>` until a
publisher revision is established. Then give the exact worksheet name,
Excel row or cell range, and any Arm condition/case/rule identifier.
For example, coordinates are placeholders:
`[DDI0625, revision not stated, 'Exc entry', <cell range>]`.
Excel row numbers are physical worksheet rows, distinct from any
logical case numbering. A matching row identifies a **permitted**
outcome; when several rows match, preserve the set of permitted
outcomes. Do not infer a unique result or flatten hierarchical headers.

In source comments, prefer the shortest complete form with an exact
section and revision; experimental procedures and evidence should add
page, rule, register field, or workbook cell where material. In
ChatGPT/Codex exchanges, use the same IDs and state the file digest
when distinguishing two nominally identical local copies.

## Supplemental fault-spreadsheet navigation

Both XLSX files were inspected as Open XML archives: workbook and sheet
relationships, shared strings, cell coordinates, merged ranges, cover
drawing text, and core/custom metadata. This preserves the two-row
hierarchical headers and merged cells; no decision table was copied
into the repository. The worksheets contain formulas/formatting and
color cues that must be checked in the original workbook when a case
is used. The local files are readable, but their official revision is
not explicitly established.

| ID; exact worksheet | Purpose and navigation |
|---|---|
| DDI0624 `Cover` | Cover drawing identifies *Armv8-M Faults on Instruction Fetch*; worksheet has no data cells. |
| DDI0624 `Introduction` | `B4:G8` revision history; `B19:D23` terms; `B25:B30` scope. The history at row 7 has Excel date 29 Jul. 2024 and says first release for EAC. |
| DDI0624 `V8MFETCH` | `A1:A3` title, permissibility and left-to-right priority notes; `A4:X4` conditions and outcome headings; `A5:X51` candidate rows. Column A names an outcome; B–X describe security, halfword fetch, MPU/bus/debug and instruction-state conditions. |
| DDI0625 `Cover` | Cover drawing identifies *Armv8-M Faults on Exception Handling*; worksheet has no data cells. Its abstract mistakenly describes instruction fetch. |
| DDI0625 `Introduction` | `B3:F7` revision history (row 6: 29 Jul. 2024, first release for EAC); `B32:B39` outcome/priority notes; `B41:F46` worksheet flow, including escalation; `B48:C51` color notes/legend. |
| DDI0625 `Exc entry` | `A1:X3` merged hierarchical headers (general, integer stacking, FP state/stacking, vector fetch); `A4:X57` outcomes and input-condition rows. |
| DDI0625 `Exc return pre tail-chain` | `A1:N2` merged headers for EXC_RETURN and FP conditions; `A3:N25` candidate rows. |
| DDI0625 `Exc return post tail-chain` | `A1:X3` merged headers for integer/FP unstacking and return checks; `A4:X31` candidate rows. |
| DDI0625 `Exc entry - tail-chain` | `A1:N3` merged general, stacking and vector-fetch headers; `A4:N27` candidate rows. |
| DDI0625 `FP lazy state preservation` | `A1:M3` merged state/stacking headers; `A4:M14` candidate rows. |

DDI0624 `V8MFETCH!A2` says more than one row can match and any one
matching outcome is permitted; `A3` specifies left-to-right event
priority when filtering. DDI0625 `Introduction!B36:B39` says the same
for derived faults, adds that no matching row means no derived fault,
and explains configuration-dependent target state. Its
`Introduction!B41:F46` defines worksheet flow order. Follow these
notes before selecting rows; do not use a spreadsheet filter that
silently discards earlier-priority conditions. Color-coded notes at
`Introduction!C49:C51` flag asynchronous DWT/BusFault recognition
and same-access conditions. The color-to-row mapping was not rendered
or validated in this catalogue; inspect the workbook's visual styles
for a case-specific analysis.

The workbook covers say **8 March 2024**. Their `Introduction`
revision-history cells contain Excel serial 45502, or **29 July 2024**.
Core metadata records modification on **24 February 2026**, which is
not a release date. DDI0624 also has a custom property `Current
Version=0.3`, but it is not presented as an official document issue;
DDI0625 has no corresponding version property. Do not cite a guessed
revision. These inconsistent dates and missing revision identifiers
need publisher or document-control resolution.

## Accessibility, discrepancies, and limits

- **Inventory:** all 16 expected document IDs have one readable local
  file. Fourteen PDFs yielded page counts, metadata, text and outline
  navigation; representative PM0264, RM0456 and ES0553 register,
  memory, and errata tables were text-inspected. Both workbooks yielded
  named sheets, cells and merged-header structure. ARM-DGUI and ARM-TRM
  carry PDF encryption flags, but the local parser could read their
  metadata, text and outlines without a supplied password. No file
  was inaccessible for this catalogue.
- **PDF review depth:** cover/release pages, contents/bookmarks, and
  selected relevant pages were inspected. No PDF was reviewed in full.
  Some mathematical glyphs, table layout, and diagrams may not survive
  text extraction. Graphical rendering was not performed; inspect
  original pages when visual structure matters.
- **Revision mismatch:** the prior FLT-01-001 procedure cites RM0456
  Rev. 6; this local file is Rev. 7. Historical citations remain tied
  to their original revision. This catalogue does not retroactively
  re-evaluate the experiment.
- **Publication-date mismatch:** local ES0553 Rev. 6 prints June 2026;
  ST's HTML catalogue has shown July 2026 for Rev. 6. The exact local
  PDF digest above distinguishes this copy. No release date was
  substituted by assumption.
- **Workbook ambiguity:** DDI0625's cover abstract describes
  instruction fetch, contrary to its cover title and exception-handling
  worksheets. Both workbooks have unresolved issue identifiers and
  differing cover/history dates. The workbook summary is navigation,
  not a correctness review of its rules.
- **Processor applicability:** ARM-DGUI and ARM-TRM are r0p4 editions;
  ES0553 points to r0p4 core documentation, but target CPUID and
  AT623/AT624 variant remain to be independently checked. ARM-ARM's
  v8.1-M/optional material requires per-feature applicability checks.
- **URLs:** publisher sites are identified in the documents. Direct
  download URLs, current web availability, and file authenticity were
  not established from the local copies.

## Reference maintenance

1. Keep original copyrighted files in ignored `Manuals/`; verify
   `git check-ignore`, `git ls-files --stage`, and staged status before
   adding a catalogue entry. Do not force-add, extract into tracked
   directories, or publish the files.
2. Assign a stable ID once. On replacement, record the new exact title,
   issue/date, filename, byte count, page count, and SHA-256. Retain
   historical revision/digest information needed to interpret earlier
   citations and evidence.
3. Compare the new document with the cited revision for changes that
   could affect requirements, procedures, implementation assumptions,
   or conclusions. Flag those artifacts for human review. Do not
   automatically rewrite historical citations to a newer revision.
4. Review document revision changes as configuration changes. Record
   discrepancies, accessibility limits, and approval status separately
   from local file identity. Update this index only after checking the
   relevant content and provenance.

This catalogue and its navigation are AI-assisted engineering work and
remain unapproved until accepted through the project's review process.
