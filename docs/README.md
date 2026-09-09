# Documentation

Start with [getting started](GETTING-STARTED.md) to build the application, author a document, and construct a specification you can export and run.\
The [root README](../README.md) is the short entrance; this index routes to deeper material by the work you want to do.

## Choose a workflow

| Workflow | Guide | Contract | Evidence |
|---|---|---|---|
| Edit and resume JSON | [Authoring](authoring/USE.md) | [Sessions and requests](authoring/SESSION-CONTRACT.md) | [Authoring results](authoring/CLI-002.md) |
| Construct a CLI and explore mocks | [Composition](composition/USE.md) | [Definitions and bindings](composition/CONTRACT.md) | [Composition results](composition/CLI-003.md) |
| Inspect a CLI as a tree | [Platform example](composition/examples/platform/README.md) | [Run presentation](run/CONTRACT.md) | [Run results](run/RESULTS.md) |
| Author a schema and constrain an instance | [Constraints](constraints/USE.md) | [Supported schema semantics](constraints/CONTRACT.md) | [Constraint results](constraints/CLI-004.md) |
| Export, import, and use a specification separately | [Specification transfer](authoring/IMPORT-USE.md) | [Document import](authoring/IMPORT.md) | [Import results](authoring/IMPORT-RESULTS.md) |
| Assemble reusable local components | [Components](components/USE.md) | [Assembly](components/CONTRACT.md) | [Component results](components/RESULTS.md) |
| Attach a granted JSON file read | [Connected reads](connected/USE.md) | [Authority and observations](connected/CONTRACT.md) | [Connected results](connected/RESULTS.md) |
| Use configured verbs directly | [Run mode](run/USE.md) | [Routing, output, and persistence](run/CONTRACT.md) | [Run results](run/RESULTS.md) |
| Prepare another actor to continue | [Reuse](reuse/USE.md) | [Continuation experiment](reuse/CONTRACT.md) | [Fresh-agent trial](reuse/FRESH-ACTOR-TRIAL.md) |
| Reproduce measured workflow costs | [Comparison](reuse/COMPARISON-USE.md) | [Fixed comparison](reuse/COMPARISON-CONTRACT.md) | [Latest comparison](authoring/IMPORT-RESULTS.md) |

---

## Understand the boundaries

| Available behavior | Boundary |
|---|---|
| Contextual authoring preserves typed JSON values and exact number tokens. | Documents, depth, requests, and checkpoints have explicit [bounds](authoring/USE.md#bounds-and-current-scope). |
| An authored schema can govern another document. | The [selected dialect](constraints/CONTRACT.md#interpretation) is a bounded local subset; unsupported features are rejected. |
| CLI definitions describe nested contexts and scalar command signatures. | Optional parameters, named flags, and arbitrary argument grammars are not implemented. |
| Explicit mocks can show proposed functionality. | Simulation is labeled; a mock is not an external observation. |
| A connected command can read a granted local JSON file. | There is no network, executable-hook, or mutation provider; authority is supplied for each process. |
| Components assemble with checked dependencies and isolated state. | Nested assemblies, shared mutable state, and version negotiation need new contracts. |
| OpenAPI descriptions can be authored as data. | Automatic API command generation and invocation remain held on the [board](BOARD.md#held). |
| One fresh-agent continuation passed its fixed task. | Comparative agent effort or token savings have not been established. |

---

## Change the project

| Record | Purpose |
|---|---|
| [Vision](../VISION.md) | Enduring owner intent and separate success dimensions. |
| [Architecture](ARCHITECTURE.md) | Target responsibility model and explicit contract boundaries. |
| [Board](BOARD.md) | Current triage, dependencies, and decisions. |
| [Backlog](BACKLOG.md) | Durable findings, dispositions, and revival triggers. |
| [Contributing](CONTRIBUTING.md) | Build and verification workflow for a source change. |
| [Resume work](resume-work.md) | How a new contributor or agent recovers the relevant state. |
| [Scaffold workflow](scaffold-workflow.md) | Regeneration and checking of the layer declaration's views. |
| [Publication](PUBLISHING.md) | Source distribution and remaining release decisions. |

---

## Historical records

The [original discussion](context/README.md), approvals, acceptance tasks, design audits, and [execution evidence](evidence/) retain their original context.\
Implementation results link to the checks for each selected increment.\
Current guidance can be corrected; a completed task or failed experiment must not be rewritten to make a later implementation appear to have passed it.

Historical paths, toolchain identities, and checkpoint hashes identify the environment that produced that evidence.\
They are not install instructions for another machine.\
Checkpoints identify their exact declaration bytes; portable interface exports are the supported way to transfer an interface when those declarations change.\
The [publication review](publication/REVIEW.md) gives the fresh source-consumer observations separately.

---

## Mechanics, rationale, and consequence

### Mechanics

Choose a workflow, read its contract when a guarantee matters, and follow the matching evidence before asserting that guarantee.

### Rationale

A small entry surface can remain useful while the complete engineering record stays recoverable.

### Consequence of violation

Readers can lose a capability among historical updates or mistake a saved result for a current measurement.
