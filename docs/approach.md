# Working approach

**Status: provisional approach derived from the recorded discussion.**
The [vision](../VISION.md) states purpose, and the [target architecture](ARCHITECTURE.md) records the proposed responsibility boundaries.
This document describes how the value claims can become reviewable work and measurable experiments.

## Construct in context

An author begins from an existing artifact or an empty document.
The interaction exposes the current location, available structure, relevant constraints, pending changes, and unresolved findings.
Meaningful operations progressively produce the artifact.
Incomplete work remains inspectable until its selected acceptance conditions hold.

The discussion takes navigation such as `edit`, `up`, and `top`, and authoring operations such as `set`, `delete`, `show`, and `commit`, as inspiration from VyOS.
Those names illustrate the desired contextual experience; this document does not fix their syntax or claim an implemented command surface.

A user need not manually edit the serialized JSON representation to complete the intended authoring workflow.
Direct import and export can remain useful without becoming a hidden requirement for finishing the task.

The same workflow covers ordinary JSON documents, schemas, API descriptions, and the CLI's own definitions.
The original idea is a construction workflow whose value is measured independently of the amount of specification it produces.

---

## Give a document an explicit role

| Role | Example | What that role establishes |
|---|---|---|
| Authored data | A JSON Schema being constructed. | The document is editable structured content. |
| Active constraint | That schema governing the construction of an instance. | Its supported interpretation constrains or evaluates the instance. |
| Interface definition | A declaration of contexts, operations, and expected outcomes. | A CLI surface can be derived from that declaration. |
| Mocked interface | A definition with declared simulated behavior. | Intended interaction can be explored before real functionality is attached. |
| Connected interface | A definition with explicit real behavior bindings. | Invocations can reach capabilities within their declared authority and guarantees. |

These are uses of artifacts, not an assertion that every artifact passes through a single mandatory lifecycle.
Editing a definition does not implicitly activate it.

JSON Schema provides meta-schemas for checking the structure of other schemas; the selected dialect still determines interpretation.
See the [JSON Schema explanation](https://json-schema.org/understanding-json-schema/reference/schema).

OpenAPI describes HTTP API interfaces, including their operations.
Authoring an OpenAPI description is distinct from interpreting it as a CLI or carrying out its HTTP behavior.
See the [OpenAPI specification](https://spec.openapis.org/oas/latest.html).

---

## Close the authoring loop

The proposed demonstration begins with the generic authoring experience and a description of valid CLI definitions.
An actor constructs a new CLI definition through contextual operations, inspects it, activates it, and exercises the resulting interface.
The actor can return to authoring to refine the definition.

The initial tool implementation necessarily bootstraps the capabilities needed for this loop.
Hand-authored bootstrap material is identified as such and is not reported as proof that self-authoring already works.
Once the loop exists, examples generated through it provide a concrete test of its completeness.

The demonstration records both successful and rejected operations.
An actor that constructs its own rules also checks the resulting behavior against requirements that were fixed outside those rules.

---

## Explore intended functionality before attaching it

A proposed CLI makes operations, inputs, outcomes, and unresolved behavior discoverable.
Mocked interaction can expose omissions or awkward workflows before real functionality is implemented.

The [CLI-003 model](composition/CONTRACT.md#bindings) selects both example responses and bounded simulated state changes for the service-catalog interaction.
Broader mock behavior remains open to later concrete use cases.

Unbound behavior is explicit.
A fabricated success response cannot substitute for a missing implementation.
Once a real capability is attached, the evaluation checks its observed behavior against the declared expectations and original task.

---

## Carry enough structure to resume

The CLI definition can provide a navigable account of the target system's relevant entities, operations, constraints, and relationships.
Ownership inside a JSON tree is only one possible relationship.
Explicit references preserve relations that do not fit that ownership structure.

A continuation experiment considers the following information:

| Information | Why the next actor needs it |
|---|---|
| Original intent and acceptance expectations | Recover what the work is meant to accomplish. |
| Definition identity and dependencies | Interpret the intended interface consistently. |
| Working document, location, draft, and accepted baseline | Continue edits from the actual work state. |
| Applied schemas and their interpretation context | Recover which constraints govern the next change. |
| Decisions, rationale, and unresolved questions | Avoid silently redesigning a settled point or treating an open point as settled. |
| Binding and capability availability | Distinguish inspectable promises from usable behavior. |
| Invocation outcomes and effect evidence | Determine what happened and what remains uncertain. |

This is a content requirement to investigate, not a prescribed archive format.
Portability includes explicit missing dependencies and capabilities; it does not assume they exist in every destination environment.

---

## Agent operating discipline

- Discover available operations and relevant constraints before inventing syntax or behavior.
- Inspect state and revision context before acting, and refresh it when it changes.
- Preserve the original requirements when authoring schemas, mocks, and acceptance examples.
- Use deterministic operations for repeated mechanical work.
- Read failure details and mutation outcomes before choosing repair or retry.
- Distinguish validated data, simulated responses, invocation success, and observed effects.
- Preserve decisions and evidence in artifacts another actor can inspect.
- Reuse an existing definition when it fits the task, while retaining its assumptions and dependencies.
- Treat self-configuration as work within existing authority.

---

## Evidence journeys

| Journey | Completion predicate | Failure that must be exercised |
|---|---|---|
| Structured authoring | The requested JSON document is constructed through supported operations with the intended values and relationships. | Unusual keys, array changes, string-like booleans, rejected edits, and incomplete drafts. |
| Schema construction | A schema is authored, then used to distinguish externally specified valid and invalid instances. | An incomplete schema, unsupported interpretation, and a mistaken constraint that is internally consistent. |
| CLI self-authoring | A definition is constructed through the authoring CLI and produces its intended discoverable interface. | Invalid activation, missing reusable components, and absent behavior bindings. |
| Specification-driven mocking | An actor can inspect and exercise declared interaction while all simulated behavior remains identifiable. | A missing operation and an apparent success that lacks real effect evidence. |
| Resumption | A fresh actor extends the work from recorded artifacts and the original task without the originating conversation. | Stale context, unavailable dependencies, and unresolved operation outcomes. |
| Human and agent parity | Equivalent semantic requests yield equivalent interpretation and relevant results. | Presentation differences that conceal constraints or change the target of an operation. |
| Cross-project reuse | Independently scoped definitions compose with explicit compatibility behavior. | Name collisions, conflicting constraints, and incompatible dependencies. |

The [authoring journey](authoring/CLI-002.md) and [CLI construction journey](composition/CLI-003.md) record the first selected implementations and their evidence.
Remaining journeys keep their own task, contract, and acceptance requirements; directory scaffolding does not complete them.

---

## Evaluate workflow value

Report correctness, completion, repair, resumption, reuse, and human intervention separately.
Record total construction and execution effort alongside artifact volume, elapsed time, and token consumption where those are observable.
Do not collapse the dimensions into one score or treat fewer lines of configuration as a success gate.

Use credible comparison workflows, including targeted document editing with validation and conventional CLI construction.
Compare tasks and acceptance conditions consistently, and account for whether a reusable definition already exists.
Separate the cost of constructing a reusable asset from the value of later use without excluding either from the record.

Claims of universality, language superiority, automatic atomicity of external effects, or a particular percentage of token savings are not established by the supplied prior conversation.
They require evidence scoped to the actual implementation and workload.

---

## Mechanics, rationale, and consequence

### Mechanics

Use concrete authoring, mocking, continuation, and reuse journeys to turn the vision into observable outcomes.
Keep unresolved interpretation choices visible in the architecture rather than guessing their answers inside a demonstration.

### Rationale

The valuable deliverable includes the way an artifact is constructed and the understanding it preserves.
An evaluation that inspects only the final document misses that claim.

### Consequence of violation

A compact output, internally consistent mock, or short command trace can look successful while failing the owner's intended workflow.
