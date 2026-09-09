# Publication preparation review

**Later update:** the owner selected [MIT](../BACKLOG.md#decision-0002-use-the-mit-license) after this review.\
The review body and its evidence below retain the state measured before that decision; [current release guidance](../PUBLISHING.md) tracks the remaining choices.

**Status: local preparation complete; release decisions remain held.**\
Identity: `CLI-009`, created in the [backlog](../BACKLOG.md#cli-009).\
The project owner's request is: "Ok, I want to prepare this repo for publishing - that is a full mission-kit sweep and documentation set (VISION, README, BOARD, ARCHITECTURE)".

## Scope and authority

This is an engineering self-review of source publication readiness.\
It covers the existing Rust application, local build and test tooling, declared operations, user workflows, document roles, evidence retention, dependency metadata, and the files a new source consumer receives.\
It prepares a source repository; package-registry publication, repository hosting, licensing, and actual release remain explicit owner decisions.\
The existing vision and selected implementation contracts govern the work.\
No new product architecture, provider, or CLI grammar is selected by an editorial sweep.

The review uses the mission-kit ledger on canonical `main`, with the observed revision `4975159283181f0c5f40e0364d4d7894836dd564`.\
It applies `M3` default-reject discipline, `M4` history preservation, `M2` literal documentation execution, `M7` axiom interrogation, `AR1`/`AR3`/`AR6` artifact roles, and the applicable style rules.\
The existing artifact set predates this request; the owner's request authorizes completing the set without restarting artifact bootstrap or repeating prior product approvals.\
The OSS adoption research workflow in `K3` is not applied to this already familiar implementation.

No roster-distinct verifier is assigned to this review.\
Local commands and this author's source inspection are executor evidence, not independent certification.\
Original owner intent, unchanged upstream schema cases, source mechanism, and local execution serve different questions; they do not become four independent whole-system assurances merely by being listed together.

---

## Measured starting state

| Surface | Observation |
|---|---|
| Repository identity | Standalone repository; initial `HEAD` is `50a2b4dec47bbd9f1124f0043e6d21406e7ad4a3`. |
| Tracked content | Only `AGENTS.md` is committed; application, tests, tooling, and documentation are untracked working content. |
| Distribution | No configured remote; `Cargo.toml` has `publish = false`; no project license. |
| Documentation | The README mixes user journeys with implementation chronology; the target architecture includes successive implementation updates; some continuation guidance still calls completed capabilities future work. |
| History | Existing evidence occupies about 61 MiB as reported by `du -sh`; original task, approval, correction, and failure records are retained. |
| Recovery before editing | A complete pre-edit working-content archive is retained locally at `target/publication-prep/before/repository.tar.gz`, SHA-256 `50b7d8d6702a346ee11825d0254bfaa5f9b7a74784d75604b8f7e0ec208a5111`. |

The source archive is a local recovery aid, not a published artifact or a substitute for a future commit.\
Existing historical evidence is not rewritten to make it look reproducible on a different machine or under a new declaration digest.

---

## Acceptance predicates

| Predicate | Observation that fails it |
|---|---|
| The documentation retains owner intent and makes its authority explicit. | Universal coverage, actor savings, external effects, or architectural ratification are asserted without supporting evidence. |
| A new consumer can install and use the source with documented prerequisites. | A literal command needs an unmentioned file, machine path, flag, dependency, or repair. |
| The consuming CLI exposes direct configured verbs. | The introductory workflow requires `invoke` or authoring-session setup to call an exported spec. |
| Contributors can reproduce application and document verification. | A stated check fails, depends on retained local build products, or confuses fixture consistency with application behavior. |
| The release surface is reviewable. | A license or remote is invented, hidden runtime artifacts are selected, sensitive-content findings have no disposition, or build inputs are absent from the source copy. |
| The board and backlog retain every existing record and agree. | An identifier disappears, state differs, a dependency cycles, or held work loses its observable trigger. |
| Frozen evidence remains intact. | A prior task, approval, oracle, failure, or verification artifact changes without an explicit preservation disposition. |

---

## Findings and verification

Eleven improvement candidates were considered: six accepted and completed, three held, and two rejected.\
The yield is one repository ignore-rule fix plus documentation and source-release preparation; no runtime bug fix or product feature is claimed.

| Finding | Evidence | Disposition |
|---|---|---|
| F01: The README mixed entry journeys with completion chronology. | Compare the pre-edit archive with the [README](../../README.md). | Fixed: an experimental-status pitch and install/use/test/remove paths lead to focused guides. |
| F02: The target architecture mixed successive implementation instants into its model. | The preserved original and the [architecture](../ARCHITECTURE.md). | Fixed: one target instant, section maturity, responsibilities, interfaces, lifecycle, verification, and an explicit open register. The generated map is retained. |
| F03: Current guides called completed assembly and continuation future work. | [Authoring correction](../authoring/USE.md#bounds-and-current-scope), [composition guide](../composition/USE.md), and [continuation correction](../resume-work.md#corrections-retained). | Fixed: current scopes link to their measured results; withdrawn statements remain identifiable. |
| F04: Only the root Cargo build directory was ignored. | The original `.gitignore` contained `/target/`; `git check-ignore` now matches both the root and scaffold build products. | Fixed: `target/` covers Cargo packages at every depth. |
| F05: Source release inputs and the consumer entrance were incomplete. | [Cargo metadata](../../Cargo.toml), [toolchain declaration](../../rust-toolchain.toml), and [getting started](../GETTING-STARTED.md). | Fixed: descriptive package metadata, a default executable, an explicit toolchain, source prerequisites, and an executed author/export/run journey. |
| F06: Contributor verification and release responsibilities were scattered. | [Contributor guide](../CONTRIBUTING.md), [publication guide](../PUBLISHING.md), and [CI workflow](../../.github/workflows/check.yml). | Fixed: reproducible local checks and a prepared read-only CI workflow. Hosted execution remains unmeasured. |
| F07: Licensing, hosting, and publication are unselected. | No project license or configured remote; both packages retain `publish = false`. | Held in [CLI-010](../BACKLOG.md#cli-010), with specific owner decisions and a release trigger. |
| F08: Strict Clippy fails before completing an all-target inventory. | [Raw strict-lint output](../evidence/publication/strict-clippy.log). | Held in [CLI-011](../BACKLOG.md#cli-011): 131 large-error diagnostics, seven collapsible conditions, and one byte-string suggestion. No error protocol is changed or warning globally suppressed. |
| F09: Current architecture is not derived from verified exit criteria. | [Renderer](../../tools/scaffold/src/main.rs) versus the `AR1` current-state requirement. | Held in [CLI-012](../BACKLOG.md#cli-012); the target view is not mislabeled as a generated current architecture. |
| F10: Removing historical paths or failed experiments would make the tree smaller. | About 61 MiB of original evidence and the retained source discussion. | Rejected: preserve evidence intact and provide focused navigation. Public disclosure remains explicit release scope. |
| F11: A publishing sweep could expand grammar, hooks, package resolution, or split public crates. | The [open register](../ARCHITECTURE.md#owed-and-open-register) contains no newly selected consumer for those changes. | Rejected: leave product behavior unchanged and retain concrete-consumer triggers. |

### Executed verification

| Check | Result and evidence |
|---|---|
| Clean-copy normal application suite | `cargo test --locked --all-targets`: 70 tests passed in the [raw log](../evidence/publication/normal-tests.log). |
| Clean-copy instrumented suite | `cargo test --locked --all-features --all-targets`: 87 tests passed, including actual terminal interaction and fault cases, in the [raw log](../evidence/publication/instrumented-tests.log). |
| Clean-copy documentation suite | 13 tests passed in the [raw log](../evidence/publication/source-document-tests.log); generated views also [matched](../evidence/publication/source-generated-check.log). |
| Production build | The [fresh source build](../evidence/publication/source-release-build.log) and installed binary both have SHA-256 `8a9fc5c1d30fa8a20e18982597b9923eb1d4f28875ae9ad4b63d912f2605d99c`, matching the measured existing normal release executable. |
| Literal user workflow | Getting-started shell blocks passed; [stdout](../evidence/publication/getting-started.stdout) and [command trace](../evidence/publication/getting-started.stderr) retain authoring, reopen, export, direct use, simulation labels, and source-free named-session status. |
| Install, use, and remove | [Cargo installation](../evidence/publication/source-install.log) succeeded with an isolated installation root; [outside-checkout invocation](../evidence/publication/installed-quota.json) returned the expected simulated result. [Uninstall](../evidence/publication/uninstall.log) removed the binary, and [clean](../evidence/publication/source-clean.log) removed build products while example checkpoints and exports remained. |
| Formatting and standing context | Application and scaffold formatting passed. The canonical standing-context checker passed including URL resolution; [output](../evidence/publication/standing-context.log). Canonical S6/S8/S10/S12/S13 checks cover the twelve current documents rewritten in this sweep; historical records are not restyled. |
| Dependency advisory lookup | The [query and response count](../evidence/publication/dependency-check.json) records 159 package/version queries, 159 responses, no pagination remaining, and zero returned matches. Both package lockfiles' registry dependencies are covered. The [OSV batch contract](https://google.github.io/osv.dev/post-v1-querybatch/) defines the lookup. |
| Preservation and final document reconciliation | The [verification record](../evidence/publication/verification.json) identifies source and record preservation checks, final board correspondence, and the final file manifest. |

These checks ran on the same local Linux host using the declared Rust toolchain and its dependency cache.\
The source-copy build used a fresh target directory, not a retained build product; it was not a second machine or a fresh operating-system installation.\
The instrumented continuation target took 409.89 seconds in this unoptimized run, so the contributor guide explicitly allows several minutes for the full experiment suite.

### Diagnostic and assurance limits

The first strict-lint attempt required installing the missing Clippy component.\
The subsequent command `cargo clippy --locked --all-features --all-targets -- -D warnings` failed with the retained diagnostics; no clean-lint claim is made.

The current `cargo-audit` installation failed because its dependency build rejected this host's C compiler; the [build failure](../evidence/publication/advisory-tool-build-failure.log) is retained.\
A compatible older auditor installed but could not parse a current advisory's CVSS version; the [parser failure](../evidence/publication/advisory-parser-failure.log) is retained.\
The direct OSV query is a separate named instrument, not a successful `cargo audit` run.\
It checks known package/version advisories, not yanked or unmaintained status, project-source vulnerabilities, or completeness of all security knowledge.

The credential-pattern scan searched private-key headers, AWS access-key forms, GitHub token forms, OpenAI-style key forms, and Slack token forms across the intended repository surface; no matching files were returned.\
A separate filename check found no conventional private-key or environment-secret files in the selected file list.\
These bounded scans do not certify the absence of every secret or determine whether publishing the conversation and machine-path provenance is intended.

No source release, registry package, hosted CI run, independent verifier attestation, broader platform support, or universal agent-efficiency claim results from this preparation.

---

## Mechanics, rationale, and consequence

### Mechanics

Read the implementation and its contracts, refresh current navigation documents, execute their commands in an isolated source copy, and retain raw results with source identity.

### Rationale

A source release needs an understandable entrance and reproducible checks without erasing the experiments that establish its current limits.

### Consequence of violation

A new user can mistake a target or mock for a supported capability, while a new contributor can mistake a locally passing workspace for a reproducible source distribution.

---

## Axiom sweep

**Verdict: pass-with-guardrails for local preparation; blanket mission-kit conformance and independent release assurance are not claimed.**\
The review covers every current axiom rather than selecting only favorable ones.\
The local kernel is stateful and configuration-driven; an LLM can consume it, but it neither hosts a model nor coordinates autonomous agents.\
Applicability follows those properties, not the use of the word "agent" in the vision.

| Axiom | Weight here | Measured alignment and explicit limit |
|---|---|---|
| A1 state transparency | Load-bearing for persistent sessions | [Session storage](../../src/session_storage.rs) and the retained recovery tests expose candidate, accepted baseline, revision, and latest receipt. Named sessions persist; temporary sessions and process grants intentionally do not. The stronger mandate of all state surviving infrastructure restart is not met by an unnamed run and is not claimed. |
| A2 isomorphic specification | Load-bearing for configured interaction | [Operation definitions](../../src/operation_definition.rs) and [run routes](../../src/cli_run_routes.rs) drive dispatch and discovery. Activation and schema attachment are explicit snapshots, not automatic desired-state reconciliation. The responsibility renderer does not derive current architecture; [CLI-012](../BACKLOG.md#cli-012) retains that gap. |
| A3 composition | Load-bearing | The [registry](../layers.json) assigns one duty per responsibility and the [source map](../CONTRIBUTING.md#source-map) identifies internal code boundaries. A universal stable crate API or plugin protocol has not been earned by a consumer. No speculative split or error-protocol refactor is included in this sweep. |
| A4 knowledge fidelity | Load-bearing | Original tasks, approvals, oracles, failures, and evidence are retained; current guides route readers to them. The shortened entry documents are navigation projections, not replacements for source records. The four document roles separate purpose, use, target design, and work selection. |
| A5 perceptual parity | Load-bearing at the human/agent boundary | Paired application tests and direct-run tests exercise shared operation semantics and persistent simulation labels. They do not measure a universal perception-latency bound or prove that every external agent hydrates state before acting. |
| A6 collaboration | Supporting for artifact handover | Definitions and exports support continuation without retranscribing the whole document. Automatic work routing and approval cascades are not kernel responsibilities; no multi-agent coordination guarantee is inferred. |
| A7 resilience | Load-bearing for local failure behavior | Typed failures, stable locking, checkpoint publication, receipt replay, and actual interruption tests expose the supported recovery contract. The kernel does not promise autonomous repair or a durable log of every rejected request. |
| A8 integrity gates | Load-bearing | Normal, instrumented, documentation, and clean-copy workflows are checked separately. Same-author execution evidence does not physically seal the seven architectural layers or satisfy a roster-distinct verifier gate. |
| A9 adverse-condition evidence | Load-bearing before release claims | The existing fault suite exercises process and filesystem boundaries. Hardware power loss, distributed chaos, production telemetry, and hosted CI execution are not measured by this preparation. There is no production-deployment claim. |
| A10 autonomous evolution | Not materially implicated in product behavior | This local tool does not host a self-directing actor or autonomously create remediation work. Observed preparation friction is retained in findings and backlog instead of being concealed by a clean headline. |
| A11 deterministic work | Load-bearing for agent use | Parsing, tree edits, validation, route projection, and consistency checks execute in Rust. The runtime does not infer JSON through model text. Per-actor token savings remain unmeasured; the source discussion's unsupported numerical claims are not adopted. |
| A12 precise context | Load-bearing at the presentation boundary | Contextual discovery, bounded documents and responses, and explicit overflow errors provide scoped inputs. The kernel does not own external prompt ordering or token accounting, so a universal context-economics claim remains unsupported. |
| A13 owner authority | Load-bearing for this engineering task | Existing direction and this request authorize preparation without repeated design approvals. License terms, repository destination, and publication itself are not synthesized from a preference or from elapsed time. |
| A14 retained learning | Load-bearing | The ignore-rule defect, stale capability guidance, strict-lint baseline, and current-architecture gap receive durable dispositions. Tooling failures are retained with the checks they prevented; no learning-payback metric is inferred from recording them. |

The bounded design audits linked from the [architecture](../ARCHITECTURE.md#axiom-alignment-and-tensions) remain decision-time records.\
This all-axiom sweep reviews the near-final publication documents; it does not backdate a pre-implementation gate for the already built application.\
The toolchain, metadata, and CI draft were prepared before this dedicated table was completed, so this record does not claim pre-implementation M7 sequencing for those preparation edits.

---

## Guardrails and layered application

| Layer | Guardrail | Closeout observation |
|---|---|---|
| Intent | Preserve the construction workflow, reuse, and resumption as separate values. | Compare VISION with the retained source discussion, including the specification-volume correction. |
| Architecture | Keep one target instant and explicit section maturity. | Registry check passes; no hand-authored current projection is labeled generated. |
| User interaction | Show direct configured verbs before engineering chronology. | Execute README and getting-started commands against the source-copy executable. |
| Runtime | Do not change declaration identity or reinterpret stored outcomes for editorial convenience. | Product source and declaration bytes match the starting snapshot. |
| Knowledge | Preserve historical inputs, failed evidence, and approvals. | Compare their contents against the pre-edit archive. |
| Distribution | Include embedded declaration inputs and exclude build directories at every Cargo package depth. | A clean source copy builds and its installed binary works outside the checkout. |
| Assurance | Keep diagnostic failures separate from passing required checks. | Report exact commands, output, status, and source identity; retain strict-lint and advisory-tool failures. |
| Authority | Prepare a concrete release candidate without inventing licensing or publishing it. | No remote, tag, push, or owner license decision is inferred from this work. |

Independent whole-system review remains outside the evidence produced by this sole-author preparation.\
The actual source-release record [CLI-010](../BACKLOG.md#cli-010) must resolve its own release decisions and any assurance requirement before using preparation as a release claim.
