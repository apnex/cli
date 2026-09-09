# Programmable CLI backlog

**Status: CLI-010 GitHub source bootstrap and public visibility are complete. Local preparation and prior selected implementation scopes remain closed; three follow-ups are parked.**
The [board](BOARD.md) proposes ordering and dependencies.
This record retains findings and their disposition when that ordering changes.

## Row contract

Add a record when source intent, an open architectural question, or an observed failure identifies work that is not complete.
Each record has a stable identifier, a finding, source evidence, state, and an observable reconsideration trigger.
The identifiers below are created here; they are local project records, not external issue numbers.

Keep the original finding and evidence when changing a disposition.
Append the reason and evidence for a state change; do not delete or reuse an identifier.
Retain a correction under an explicit correction banner when the original finding was wrong.

| Record state | Meaning | Board state |
|---|---|---|
| Open | Work remains in the proposed sequence. | Ready when prerequisites are satisfied; otherwise waiting. |
| Parked | Work is retained outside the proposed sequence until its trigger fires. | Held. |
| Closed | A cited change or ruling resolves the recorded scope. | Done. |

A trigger returns a record to triage; it does not authorize or automatically resume implementation.
The [board-record contract](BOARD.md#the-contract-between-board-and-record) owns the correspondence between these records and board items.

---

## Open

No selected release or implementation work remains open.
The release record below retains its earlier holds, selection, and completion; broader follow-ups remain parked.

---

## Closed preparation

### CLI-009

| Field | Record |
|---|---|
| Finding | The owner requested publication preparation and a full mission-kit review of the repository and its VISION, README, BOARD, and ARCHITECTURE documentation. |
| Evidence | The [publication review](publication/REVIEW.md) records the request, measured starting state, scope, findings, and acceptance predicates. |
| State | Closed for local source-publication preparation. |
| Selected scope | Prepare the source repository, refresh current guidance against implementation and original intent, retain historical records, and execute the documented consumer and contributor workflows. |
| Continuation trigger | Continue the already authorized preparation until its local checks pass and remaining publication decisions are explicit. |
| Closure evidence required | The four-document set is coherent, source-only installation and direct use succeed from a clean copy, application and documentation checks pass, and the review records exact evidence and unresolved release choices. |
| Completion evidence | The [review](publication/REVIEW.md#findings-and-verification) records 70 normal tests, 87 instrumented tests, 13 documentation tests, literal author/export/run and install/remove journeys, matching release bytes, preserved historical records, and the held lint and architecture-projection findings. |
| State history | Open during the selected sweep; closed after the local source-consumer checks completed. |
| Remaining boundary at preparation closeout | No actual release, license selection, configured remote, hosted CI result, or independent verifier assurance is claimed; publication remains CLI-010. |
| Later update | Decision 0002 under CLI-010 settles MIT licensing after this preparation closeout. |

The earlier statement "No selected work remains open" described the board before this publication request.
CLI-007 retains its concrete-consumer trigger below.

Correction: the prior status headline said CLI-005 was the latest closed item and connected hooks remained parked after the CLI-006 record had already closed its local read scope.
The corrected headline above follows the durable rows; broader providers remain unselected.

---

## Closed source release

This selected release retains the findings and decisions that preceded its authorization.

### CLI-010

| Field | Record |
|---|---|
| Finding | A prepared source tree is not yet an intentionally licensed and published repository. |
| Initial evidence | The [publication guide](PUBLISHING.md) and [preparation review](publication/REVIEW.md) retain the missing license, destination, and release decisions. |
| State | Closed for the GitHub source bootstrap and subsequent public visibility selection. |
| Initial reason held | The request authorizes preparation; no project license, remote destination, or publication instruction has been supplied. |
| Reason held after MIT selection | The owner selected MIT; the destination, visibility, release scope, intended disclosure of the retained records, and publication instruction remain open. |
| Revival trigger | The owner supplies the remaining release decisions and authorizes release of the reviewed source. |
| Selection | The owner instructed: "Bootstrap the apnex/cli remote"; the [source statement and interpretation](context/cli-010-bootstrap.json) retain the selected scope. |
| Work at selection | Commit the prepared source, bootstrap the GitHub remote, push `main`, and verify the remote revision and hosted checks. |
| Closure evidence required | Selected contents are committed, the exact release revision passes the documented checks, and the chosen destination is verified to contain those contents. |
| Bootstrap completion evidence | The [bootstrap record](publication/BOOTSTRAP.md) identifies the private remote, initial source commit and tree, matching remote clone, 70 normal tests, 87 instrumented tests, 13 documentation tests, literal consumer journey, and passing hosted CI. |
| State history | Parked after preparation and the MIT selection; opened under the repository-bootstrap instruction; closed after the source push and hosted verification passed. |
| Boundary at bootstrap closeout | Private source hosting is complete; public visibility, package/binary release, independent behavioral assurance, and broader platform qualification are not claimed. |
| Later visibility selection | The owner instructed "make it public"; the [source statement](context/cli-010-public-visibility.json) records the public visibility selection for the existing repository. |
| Public visibility completion | The [verification record](evidence/publication/public-visibility.json) records public repository metadata, anonymous access to `main`, and matching local and remote source identity. |
| Current remaining boundary | Public source hosting is complete; package/binary release, independent behavioral assurance, and broader platform qualification are not claimed. |

#### Decision 0002: use the MIT license

| Field | Record |
|---|---|
| Identifier | `0002`, created in this project record. |
| Date | 2026-09-09, UTC. |
| Status | Ratified. |
| Authority | Project owner, in the active project conversation. |
| Supersedes | None. |
| Amends | None; this settles the previously open license choice. |
| Affects | `LICENSE`, `Cargo.toml`, `tools/scaffold/Cargo.toml`, `README.md`, `docs/PUBLISHING.md`, `docs/BOARD.md`, `docs/BACKLOG.md`, `docs/ARCHITECTURE.md`, and `docs/CONTRIBUTING.md`. |

[DIRECTOR] The owner's license selection is verbatim: "MIT".\
[ARGUED] Leaving licensing unresolved or selecting another license would contradict that instruction; no comparative license evaluation is claimed.\
[MEASURED] The [MIT terms](../LICENSE) require retaining the copyright and permission notices in copies or substantial portions of the software.\
[ARGUED] Add those terms at the repository root, declare `license = "MIT"` in both Rust manifests, and absorb the decision into current release guidance without changing third-party notices or historical observations.\
[OPEN] This license choice does not supply a publication destination or authorize a release.

Future rulings amend or supersede this dated decision without rewriting its source statement.

---

## Parked

These are proposed holds for the broader sequence, not a ruling to remove the capabilities from the vision.

### CLI-011

| Field | Record |
|---|---|
| Finding | Strict Clippy fails on the current library with 139 diagnostics: 131 large-error warnings, seven collapsible conditions, and one byte-string suggestion. |
| Evidence | The [publication review](publication/REVIEW.md) retains the command and raw lint result. Compilation stops in the library, so this is not a complete all-target lint inventory. |
| State | Parked. |
| Reason held | The existing required checks do not include a zero-warning Clippy gate. Changing the error representation throughout the application is a separate implementation concern without a measured behavior defect. |
| Revival trigger | A consumer requires a strict lint gate, or measured error-path costs justify changing the error representation. |
| Closure evidence required | An explicit lint policy is selected, relevant diagnostics are resolved with preserved error semantics, and the strict command completes across every requested target. |

### CLI-012

| Field | Record |
|---|---|
| Finding | The registry generates target responsibility views; it does not derive a current architecture from verified binary exit criteria. |
| Evidence | [Architecture open register](ARCHITECTURE.md#owed-and-open-register), [layer registry](layers.json), and [renderer](../tools/scaffold/src/main.rs). |
| State | Parked. |
| Reason held | Existing contracts and results support bounded capability reporting, but no current/target projection contract has been selected. The publication sweep must not invent a hand-maintained current architecture and call it derived. |
| Revival trigger | A selected architecture change or consumer needs mechanical comparison of current and target state at the system altitude. |
| Closure evidence required | Verified exit criteria mechanically derive a current view comparable to the target, with stale and failed evidence rejected by a drift check. |

### CLI-007

| Field | Record |
|---|---|
| Finding | Authoring an OpenAPI description is in scope, but translating one into invocable CLI operations requires additional mapping and protocol behavior. |
| Evidence | [Document roles](approach.md#give-a-document-an-explicit-role) and [OpenAPI interpretation](ARCHITECTURE.md#owed-and-open-register). |
| State | Parked. |
| Reason held | No representative API task yet establishes how its operations should appear or execute through the CLI. |
| Revival trigger | A concrete API description and requested CLI interaction establish operation mapping, authentication, and response expectations. |
| Closure evidence required | The selected operations have correct discovery and mapping, with real invocation evidence supplied by a compatible capability when execution is included. |

---

## Closed

### CLI-008

| Field | Record |
|---|---|
| Initial finding | Exported CLI definitions execute only inside the authoring workspace through an `invoke` prefix; a normal direct-command run experience is missing. |
| Evidence | The owner challenged the extra invocation verb and asked for real verbs in run mode. The [original composition contract](composition/CONTRACT.md#scope-and-fixed-intent) explicitly deferred a standalone domain shell. |
| Initial state | Open. |
| Selection | The owner's ["Approved for next best action"](context/cli-008-approval.json) selects direct one-shot and interactive run mode before another provider. |
| Selected scope | Load exported specifications, route configured verbs directly, generate help and completion, preserve binding labels, provide normal output and exit codes, and reuse existing invocation and recovery semantics. |
| Closure evidence required | Real one-shot and interactive processes use the same authored definitions, preserve exact values and state, reject ambiguous routes and bad input clearly, transfer and resume optional sessions, and retain meaningful failures in process exit status. |
| Disposition, 2026-09-09 | The [run results](run/RESULTS.md) demonstrate command-authored exports consumed through direct verbs, consistent generated routes and terminal completion, optional state retention, unchanged checkpoints after failure, and observed publication recovery. The instrumented suite passed 87 tests and the normal build 70; the literal walkthrough passed. |
| State | Closed for the selected direct-run scope. |
| Closure evidence | [Verification](evidence/run-mode/verification.json), actual terminal transcript, original failing launcher test, applied invalid definitions, and fault markers satisfy the [run contract](run/CONTRACT.md). |
| Reconsideration trigger | A supported definition or invocation fails the selected routing, output, parity, persistence, or recovery contract. |
| Remaining scope | Optional/named parameters, OpenAPI mapping, further providers, and package negotiation require concrete consumers. No comparative agent-savings claim is established. |

### CLI-006

| Field | Record |
|---|---|
| Initial finding | No concrete connected capability establishes the necessary hook transport, authority, retry behavior, or effect evidence. |
| Initial evidence | [Hook mechanism and guarantees](ARCHITECTURE.md#owed-and-open-register) and [local versus external outcomes](ARCHITECTURE.md#runtime-behavior). |
| Initial state | Parked. |
| Original reason held | Selecting plugin or transport technology without a capability would decide guarantees before its consumer is known. |
| Original revival trigger | A named operation has a concrete external capability and examples establishing its inputs, effects, failure modes, and granted authority. |
| Closure evidence required | That capability is bound and its observed outcomes, failures, and unresolved effects are checked against its declared contract. |
| Selection, 2026-09-09 | The owner's [approval for next](context/cli-006-approval.json) selects the connected boundary. The implementer chooses local service-catalog inspection as the bounded reference capability; no production target or mutation authority is inferred. |
| Selected scope | The [contract](connected/CONTRACT.md), [task](connected/acceptance/TASK.md), and [audit](connected/AXIOM-AUDIT.md) establish a native JSON file reader, logical capability requirements, transient per-process grants, historical observations, source-free replay, transfer, and publication failures. |
| Disposition, 2026-09-09 | The [results](connected/RESULTS.md) demonstrate command-authored connected definitions, real file observations, explicit grant requirements, plain and assembled transfer, unchanged mock/authoring state, and checked failure and recovery behavior. The instrumented suite passed 77 tests and the normal build 61; the literal walkthrough passed. |
| State | Closed for the selected local JSON read capability. |
| Closure evidence | [Application and walkthrough evidence](evidence/connected-json-read/verification.json), the original failing activation test, applied invalid inputs, and confirmed publication fault markers satisfy the selected contract. |
| Remaining limits and triggers | Executable hooks and remote or mutating providers require a concrete operation with authority, protocol, failure, and retry requirements. A reproducible supported input violating read isolation, authority separation, observation fidelity, transfer, or recovery reopens this record. File observations do not establish service liveness, atomic source snapshots, or general external-write guarantees. |

### CLI-005

| Field | Record |
|---|---|
| Finding | Reuse, deterministic transfer, fresh-agent continuation, and workflow economics are central claims with no completed product experiment. |
| Evidence | [Separate success dimensions](../VISION.md#success-dimensions), [continuation contents](approach.md#carry-enough-structure-to-resume), and [workflow evaluation](approach.md#evaluate-workflow-value). |
| Initial state | Open. |
| Proposed move | Compose definitions for a second independently scoped task, transfer an unfinished session to an actor without the originating conversation, and compare full workflow costs. |
| Revival trigger | CLI-003 and CLI-004 produce usable definitions and constraint attachments that a second task can reuse. |
| Closure evidence required | The fresh actor completes the extension against original requirements; incompatible dependencies and stale context remain visible; correctness, effort, repair, handover, reuse, and artifact volume are reported separately, including unsuccessful results. |
| Readiness update, 2026-09-08 | [CLI-003](composition/CLI-003.md) and [CLI-004](constraints/CLI-004.md) supply authored interfaces and portable schema attachments. The second task, withheld context, and independent oracle remain to be specified for this trial. |
| Selection update, 2026-09-09 | The owner's [resumption](context/cli-005-approval.json) selects this ready item. The [experiment contract](reuse/CONTRACT.md), [second task](reuse/acceptance/TASK.md), and [design audit](reuse/AXIOM-AUDIT.md) establish preparation and acceptance obligations. |
| Selected first scope | Whole-checkpoint reuse and continuation are evaluated first. General component/dependency assembly, independent actor results, and workflow comparisons retain their closure obligations; a scripted rehearsal cannot close them. |
| Preparation disposition, 2026-09-09 | Rust preparation, a recipient recorder, an independent task oracle, and paired scripted rehearsals are retained in the [progress record](reuse/CLI-005.md) and [execution evidence](evidence/cli-005/verification.json). The untouched handover is ready for an actor; this record remains open. |
| Reuse observation | The existing catalog inspection binding reads the whole simulated document, so a queue extension is visible in that service view. Isolated component state and dependency composition remain a concrete next design question. |
| Fresh-actor authorization, 2026-09-09 | The owner's subsequent [approval](evidence/cli-005/fresh-actor/authorization.json) explicitly authorizes the prepared continuation trial. |
| Fresh-actor disposition, 2026-09-09 | One actor with no forked conversation history completed the worker-platform task through the CLI; the existing evaluator passed its first submission without parent repairs. The [trial record](reuse/FRESH-ACTOR-TRIAL.md) retains 96 input lines, all failures, extra-context reporting, auxiliary inspection work, and unchanged submission hashes. |
| Remaining scope after trial | Targeted-edit comparison, repeated-construction costs, and general component composition remain open. One successful whole-prototype continuation does not establish comparative savings or isolated fragments. |
| Comparison authorization, 2026-09-09 | The owner's [approval](context/cli-005-comparison-approval.json) selects controlled comparisons against the same task. The [comparison contract](reuse/COMPARISON-CONTRACT.md) fixes separate artifact and full-continuation verdicts before execution. |
| Comparison disposition, 2026-09-09 | All three scripted methods passed definition/runtime checks. Continuation used 57 construction commands versus 179 for rebuilding; file editing used one export command plus two edits and failed staged continuation because candidate and accepted documents remained unfinished. The [results](reuse/COMPARISON.md) retain raw external work, scoped timing, the initial test-fixture failure, and six passing final trial tests. |
| Remaining scope after comparison | Establish the boundary for importing or converting an externally edited file into the candidate, then measure equivalent full-continuation costs. General component composition remains open; the scripts establish neither comparative agent savings nor isolated component state. |
| Import selection, 2026-09-09 | The owner's [agreement](context/cli-005-import-approval.json) selects whole-document candidate import and asks for separately authored/exported/imported CLI specifications to be usable in another session. The [contract](authoring/IMPORT.md) and [audit](authoring/IMPORT-AUDIT.md) fix that scope. |
| Import disposition, 2026-09-09 | The [implemented round trip](authoring/IMPORT-RESULTS.md) passed separate terminal and machine consumers, source-free resumption, constrained repair, rejection and recovery checks. The full instrumented suite passed 63 tests and the normal suite 49. Edited-file import passed the staged-state comparison using 23 CLI inputs plus two text edits; the original file-activation method still fails staged checks. |
| Remaining scope after import | Independently reusable component assembly, explicit state ownership, dependencies, and collision behavior remain to be specified and implemented for a concrete consumer. Scripted comparison costs are measured; comparative agent performance remains unclaimed. |

| Assembly selection | The owner's [approval](context/cli-005-assembly-approval.json) selects independently authored services and queues components, explicit ownership and dependencies, collision rejection, and separate-session transfer. |
| Assembly disposition | The [component contract](components/CONTRACT.md) and [results](components/RESULTS.md) establish the bounded local resolver. The complete instrumented suite passed 69 tests and the normal suite 54; the literal walkthrough passed after source removal. |
| State | Closed for the selected local reuse, continuation, comparison, import, and first component-assembly journeys. |
| Closure evidence | The original [fresh-actor trial](reuse/FRESH-ACTOR-TRIAL.md), four-method [comparison](authoring/IMPORT-RESULTS.md), and separate [component assembly evidence](evidence/component-assembly/verification.json) satisfy their own tasks and retain failures, costs, artifact sizes, and limitations. No fresh actor was measured assembling components in this increment. |
| Remaining limits and triggers | A concrete consumer needing nested assembly, shared state, package/version resolution, or schema merging reopens the relevant composition design. Comparative agent performance remains unclaimed until an authorized comparative actor trial measures it. A reproducible supported input violating isolation, dependency, collision, fidelity, or recovery guarantees reopens this record. |

### CLI-004

| Field | Record |
|---|---|
| Finding | Schema documents and API descriptions are intended authoring targets, but supported schema interpretation and the transition from authored data to active constraints are unspecified. |
| Evidence | [Explicit document roles](approach.md#give-a-document-an-explicit-role), [schema construction journey](approach.md#evidence-journeys), and [JSON Schema coverage](ARCHITECTURE.md#owed-and-open-register). |
| State | Closed for the selected local-2020-12-v1 schema and authoring contract. |
| Proposed move | Author a JSON Schema, select its interpretation explicitly, and use it to constrain an instance; include a representative OpenAPI description as authored data. |
| Revival trigger | CLI-002 demonstrates generic authoring and concrete valid and invalid instances are available to select dialect and reference behavior. |
| Closure evidence required | Externally specified positive and negative examples are interpreted correctly; incomplete schema drafts remain editable; unsupported features are explicit; authoring an API description requires no network invocation. |
| Readiness update, 2026-09-08 | [CLI-002](authoring/CLI-002.md) supplies usable generic authoring. This item is ready independently of CLI-003; its examples, dialect, and reference policy still need to be established. |
| Selection update, 2026-09-08 | [Owner approval](context/cli-004-approval.json) selects end-to-end completion. The [constraint contract](constraints/CONTRACT.md), [task](constraints/acceptance/TASK.md), and [design audit](constraints/AXIOM-AUDIT.md) fix the selected interpretation and verification obligations. |
| Disposition, 2026-09-08 | Rust schema snapshots, contextual guidance, constrained commits, and source-independent recovery are implemented; the OpenAPI construction trace produces its predefined document as data. |
| Closure evidence | [Implementation record](constraints/CLI-004.md), [measured verification](evidence/cli-004/verification.json), [application assertions](../tests/schema_constraints.rs), and [literal walkthrough](constraints/USE.md). The upstream trial agrees on 141 positive and 168 negative cases. |
| Remaining consumers | CLI-005 can use the schema attachments and composed definitions for a fresh-actor reuse and continuation trial; broader dialects and connected API behavior remain outside this closed scope. |
| Reconsideration trigger | A supported schema, instance, or request reproducibly violates the stated validation, snapshot isolation, guidance completeness, exact-number, checkpoint, or replay guarantees. |

### CLI-003

| Field | Record |
|---|---|
| Initial finding | The self-authoring loop and a configurable interface that can be explored before real functionality is attached remain proposals. |
| Initial evidence | [Composition loop](../VISION.md#the-composition-loop), [self-authoring demonstration](approach.md#close-the-authoring-loop), and [mock exploration](approach.md#explore-intended-functionality-before-attaching-it). |
| State | Closed for the selected local definition, activation, and mock contract. |
| Initial proposed move | Construct a CLI definition through authoring operations, explicitly activate it, discover its operations, and exercise declared mock behavior. |
| Initial revival trigger | CLI-002 demonstrates generic authoring and a representative mock interaction is available to define the missing contracts. |
| Closure evidence required | A definition produces its intended interface without manual serialized edits; invalid activation preserves the prior active definition; unbound and simulated outcomes remain distinguishable through export and handover. |
| Readiness update, 2026-09-08 | [CLI-002](authoring/CLI-002.md) supplies usable generic authoring. This item is ready for selection; its representative definition, activation contract, and mock behavior still need to be established. |
| Selection, 2026-09-08 | The owner's [approval](context/cli-003-approval.json) selected this item for implementation and verification. |
| Design disposition before implementation | [Independent task](composition/acceptance/TASK.md), [contract](composition/CONTRACT.md), and [audit](composition/AXIOM-AUDIT.md) established the bounded increment and implementation began. |
| Disposition, 2026-09-08 | The Rust composition profile constructs and activates declared CLI contexts and commands, interprets explicit mocks, and transfers labeled interface state through the existing transaction and storage engine. |
| Closure evidence | [Implementation record](composition/CLI-003.md), [verification artifact](evidence/cli-003/verification.json), [application assertions](../tests/cli_composition.rs), and [reproducible guide](composition/USE.md). |
| Remaining consumers | CLI-004 establishes schema interpretation; CLI-005 still owes reusable dependency composition and independent agent continuation and workflow evaluation. |
| Follow-on update, 2026-09-08 | CLI-004 has completed its bounded schema interpretation and authoring scope; CLI-005 retains the evaluation work above. |
| Reconsideration trigger | An accepted definition or request reproducibly violates activation isolation, exact values, binding classification, declared discovery, replay, or transfer semantics. |

### CLI-002

| Field | Record |
|---|---|
| Initial finding | The repository provides a responsibility scaffold, but the contextual JSON authoring workflow has no application implementation. |
| Initial evidence | [Available project scope](../README.md), [authoring intent](approach.md#construct-in-context), and [document and session entities](ARCHITECTURE.md#entity-model-and-interfaces). |
| State | Closed for the selected local authoring contract. |
| Initial proposed move | Implement the selected generic authoring journey with inspectable context, precise edits, incomplete drafts, explicit outcomes, and the agreed save and recovery behavior. |
| Initial revival trigger | CLI-001 supplies the selected journey, concrete contracts, and implementation audit. |
| Closure evidence required | The requested document is constructed through supported operations; unusual keys, value types, array edits, rejected changes, and interruption satisfy the selected contracts. |
| Readiness update, 2026-09-08 | CLI-001 supplies the [design record](authoring/CLI-001.md), [session contract](authoring/SESSION-CONTRACT.md), [acceptance corpus](authoring/acceptance/authoring-cases.json), and [audit guardrails](authoring/AXIOM-AUDIT.md#implementation-guardrails); CLI-002 is ready and remains unimplemented. |
| Selection | The owner's subsequent "approved" selected this ready item, as recorded with its interpretation and board snapshot in the [approval record](context/cli-002-approval.json). |
| Disposition, 2026-09-08 | One unpublished Rust package implements contextual construction, declared dispatch, exact values, durable candidates and receipts, atomic batches, and terminal and machine interfaces. |
| Closure evidence | [Implementation record](authoring/CLI-002.md), [verification artifact](evidence/cli-002/verification.json), executable application assertions, and [usage guide](authoring/USE.md). |
| Remaining consumers | CLI-003 and CLI-004 can establish general definition activation, mocks, and constraint interpretation; reuse and fresh-agent workflow measurements remain CLI-005. |
| Reconsideration trigger | A reproducible accepted input violates the selected fidelity, navigation, declaration, output, or persistence contract. |

### CLI-001

| Field | Record |
|---|---|
| Initial finding | The first complete authoring journey, its acceptance corpus, and concrete operation contracts are not selected. |
| Initial evidence | [Evidence journeys](approach.md#evidence-journeys), [open contracts](ARCHITECTURE.md#owed-and-open-register), and [implementation audit obligation](resume-work.md#choose-concrete-work-from-the-open-questions). |
| State | Closed for design and acceptance specification. |
| Initial proposed move | Specify a bounded journey, independent expected results, editing and recovery semantics, and the minimum implementation choices it needs. |
| Initial revival trigger | Initial board intake, or a change to the selected journey or its acceptance expectations. |
| Closure evidence required | A selected journey with falsifiable acceptance examples, concrete request and result contracts, and the required axiom alignment audit before reusable runtime implementation. |
| Selection | The owner selected this item with "approved for next" in the [retained exchange](context/cli-001-approval.json). |
| Disposition, 2026-09-08 | The [first authoring decision](authoring/CLI-001.md#decision-0001-establish-the-bounded-authoring-contract) establishes a service-catalog journey, exact editing and recovery contracts, a bootstrap operation declaration, and minimum Rust build choices. |
| Closure evidence | [Completion predicates](authoring/CLI-001.md#completion-predicate), [acceptance corpus](authoring/acceptance/authoring-cases.json), [axiom audit](authoring/AXIOM-AUDIT.md), and [verification record](evidence/cli-001/verification.json). |
| Remaining consumer | CLI-002 must implement and run the application acceptance corpus; dependency and documentation checks do not satisfy that obligation. |
| Reconsideration trigger | A changed task, operation meaning, fidelity guarantee, or persistence boundary contradicts the selected contract or audit; reopen the affected design before accepting changed runtime behavior. |

### CLI-000

| Field | Record |
|---|---|
| Finding | The owner requested a documented vision, approach, and initial layer scaffold. |
| Evidence | [Captured initial scope](project-intent.md#recorded-direction). |
| State | Closed for documentation and responsibility scaffolding only. |
| Disposition | The local working tree contains the vision, retained discussion, approach, target architecture, layer declaration, generated responsibility records, and scaffold tooling. |
| Closure evidence | [Vision](../VISION.md), [discussion provenance](context/README.md), [architecture](ARCHITECTURE.md), [layer declaration](layers.json), and [scaffold verification workflow](scaffold-workflow.md#verify-the-scaffold). |
| Reconsideration trigger | An artifact loses fidelity to the retained source intent or a generated responsibility view disagrees with the declaration. |

---

## Mechanics, rationale, and consequence

### Mechanics

Retain each finding and its source, append disposition evidence, and reconcile its state with the board when work changes.

### Rationale

The sequence can evolve without losing why a capability was proposed, held, or considered complete.

### Consequence of violation

A reordered plan can silently discard a requirement or teach the next actor that unfinished runtime behavior was completed by documentation.
