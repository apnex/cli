# Consumer-neutral application boundary

CLI-016 implements the [owner's generalization instruction](../context/cli-016-selection.json).\
Status: complete for the selected boundary and local acceptance.\
This is an engineering self-review, not an independent verifier attestation.

## Audit scope and findings

The scope is application launch, operator help, navigation, completion, endpoint persistence, recovery, and native output views.\
The [source scan](../evidence/generalization/consumer-data-scan.json) records the measured Git baseline and the exact consumer paths, names, and ports searched across all production Rust files.

| Finding | Source and consequence | Disposition |
|---|---|---|
| HTTP is mandatory for every native application | `CliApplicationProfile.http` is required and `run_application` always constructs HTTP launch help. A local simulated CLI cannot opt out of this provider. | Make the existing HTTP profile optional; continue requiring exact grants for declared HTTP capabilities. |
| Operator presentation implies endpoint settings | `CliRunSession.operator` contains endpoint settings, and compact help/tree/completion treat every operator profile as endpoint-enabled. | Keep presentation in the validated route profile and construct endpoint settings only for applications selecting HTTP. |
| HTTP recovery can misdescribe another provider's failure | The missing-endpoint rewrite in `dispatch_run_tokens` checks the error code and selection but not the invoked provider. | Apply that recovery only to HTTP commands; preserve file-provider errors. |
| Consumer vocabulary and data | Searches of production `src/` find no AGP names, management resource paths, or fixed example ports; output columns, projections, and response requirements are declared data. | Retain that boundary and add a guard for the first consumer's name. The name scan alone does not prove neutrality. |

The existing catalog HTTP fixture and the separately authored platform mock exercise different requirements.\
They are acceptance fixtures, not evidence of a second deployed consumer.\
The provider's loopback and response bounds remain explicit; generalization does not imply new remote, authenticated, or mutating transport support.

---

## Shared and consumer responsibilities

| Owner | Responsibility |
|---|---|
| Shared kernel | Typed document authoring, context routing, help and completion, sessions, provider execution, response checks, configurable projection and native table rendering. |
| Consumer definition and profile | Application identity, domain verbs, aliases, resource paths, response requirements, view expressions and columns, endpoint option/environment, and output defaults. |
| Consumer integration | Packaging under its own name and proving that configured verbs and views match its real system. |

`CliApplicationProfile.http` is optional.\
Operator presentation belongs to the validated route profile, while endpoint selection and persistence are constructed only when that profile also selects HTTP.\
An HTTP-bound definition still requires exactly matching HTTP resources.\
Existing serialized profiles remain compatible; Rust embedding interfaces remain experimental, as stated in the [contributor guide](../CONTRIBUTING.md#source-map).

The [contributor boundary](../CONTRIBUTING.md#keep-the-shared-runtime-consumer-neutral) requires contrasting behavior evidence for reusable changes and preservation of the original consumer's integration assertions.\
Examples and acceptance data may describe AGP; production dispatch must not know that name or its system model.

---

## Design audit before implementation

Verdict: pass-with-guardrails under the workspace doctrine and mission-kit A3, P4, and M7 read for this change.\
The owner's selection settles the intended boundary; no additional approval is required for its implementation.

| Principle and layer | Alignment and guardrail |
|---|---|
| A3, load-bearing composition | Presentation belongs to the route/profile layer; HTTP selection belongs to the optional provider layer. Reuse existing mechanisms without adding a general plugin framework. |
| A2 and A5, load-bearing specification and perception | Help, trees, completion, and dispatch expose endpoint controls only when configured; an unrelated provider failure must retain its own recovery. |
| A7 and A8, load-bearing integrity | Preserve endpoint persistence and grant limits. Missing HTTP configuration for an HTTP-bound definition fails before invocation. Existing AGP data must remain valid. |
| A4, supporting evidence | Keep earlier AGP acceptance records unchanged, and retain new failing probes with their corrected outcomes. |
| A13, supporting intent | Apply the owner's instruction directly and document the boundary for future contributors. |

The tension is between a universal intended substrate and evidence from one real external consumer.\
Resolve it by removing a demonstrated dependency and adding contrasting acceptance cases, without claiming all possible projects have been proven.\
Closeout must run a provider-free authored platform application, renamed catalog controls and response data, mixed-provider recovery, the source guard with an applied negative probe, existing AGP live tests, and the required shared gates.

---

## Measured acceptance

The [focused application tests and source guard](../evidence/generalization/contrasting-consumers.txt) pass.\
The [platform walkthrough](examples/platform/README.md) constructs both documents with authoring commands and runs seven commands in a real terminal.\
Its [capture](../evidence/generalization/platform-terminal.raw) retains simulation labels and has no endpoint setup; [reopening](../evidence/generalization/platform-reopened.stdout) retains the selected five replicas.\
The catalog fixture supplies its own control words, URL option, environment name, resource path, response requirements, and table cells.\
A mixed file/HTTP definition retains the file provider's failure when no HTTP endpoint is selected.

The [AGP release journey](../evidence/generalization/agp-release/measurements.json) passes without changing the consumer's definition or application profile.\
It covers root discovery, in-shell management setup, a live connection table, saved endpoint reopening, node switching, and clearing settings.\
The [full AGP gate](../evidence/generalization/agp-native-gate.txt) passes six native tests and twelve existing end-to-end tests.\
Its recipes still reproduce both embedded documents exactly.\
The rebuilt executable installed on the user's PATH passes the [same terminal journey](../evidence/generalization/agp-installed/measurements.json), including live readiness reads for the hub and `leaf.alpha`.\
The deliberate unconfigured read leaves terminal exit status 2 despite subsequent recovery; this is the expected retained error status.\
The isolated probe leaves the user's default settings file absent.

The [measurements](../evidence/generalization/measurements.json) identify the baselines, tested source manifests, installed binary digest, and counts.\
The shared gates pass 105 normal tests, 122 instrumented tests, and 13 scaffold tests, along with all three strict lint configurations, formatting, generated-document drift checks, and a normal release build.\
No production deployment, hosted CI result, independent reviewer verdict, successful IPv6 transport, or second deployed consumer is claimed.

---

## Failed probes and corrections

The [provider-free probe](../evidence/generalization/provider-free-before.txt) fails against the original mandatory HTTP field.\
The [first mixed-provider probe](../evidence/generalization/provider-recovery-before.txt) is invalid for the intended assertion: output selection fails before capability recovery.\
The [corrected probe](../evidence/generalization/provider-recovery-before-corrected.txt) requests JSON and observes the misplaced endpoint instruction before the fix.\
Both cases pass in the focused application suite after the repair.

The source guard's [negative probe](../evidence/generalization/source-guard-negative.txt) deliberately inserts the first consumer's name in a production source comment and fails.\
The [mutation record](../evidence/generalization/source-guard-mutation.json) proves the changed bytes were applied and the original digest restored.\
This checks one class of consumer leakage; the contrasting behavior cases establish the tested independence from HTTP and domain vocabulary.

The [initial scaffold check](../evidence/generalization/tests-scaffold-initial.txt) ran between adding an index link and writing its target section, so it correctly rejected a missing anchor.\
The [completed-link check](../evidence/generalization/tests-scaffold-complete-links.txt) passes all 13 scaffold tests.\
The final closeout check also evaluates the completed board and backlog.
