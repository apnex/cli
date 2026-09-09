# Project intent and initial scaffold scope

**Status: captured intent with explicitly identified implementation inferences.**

The [vision](../VISION.md) states the enduring purpose.
The [source discussion](context/design-discussion.json) carries the complete project conversation used here.

## Recorded direction

| Direction | Source | Consequence for the work |
|---|---|---|
| The resulting CLI surface and interaction derive from loaded configuration. | User message 5 | CLI meaning is declared and inspectable. |
| Future CLIs are configured or assembled from reusable pieces. | User messages 5 and 7 | Reuse remains a first-class source of value. |
| A context-aware CLI can author JSON and its own CLI definitions. | User messages 3, 5, and 7 | The authoring workflow can close the composition loop. |
| JSON Schemas and OpenAPI descriptions can themselves be authored. | User message 9 | Structured specifications are useful authored artifacts. |
| A specification can separately govern subsequent authoring. | User message 9 | Editing a document and activating its meaning remain distinct operations. |
| A CLI can be mocked before functionality is attached. | User message 9 | An unbound or mocked operation is legitimate declared state. |
| The agent-facing requirements in the preceding reply are valuable. | User message 13 referring to assistant message 12 | Discovery, precise edits, drafts, equivalent interaction meanings, recovery, evidence, and portable context shape the proposal. |
| Workflow value can outweigh specification volume. | User message 13 | Size is measured separately from correctness, continuity, and workflow utility. |
| The definition can carry a graph resembling the target system for another agent to resume from. | User message 13 | Entities, relationships, constraints, work state, and evidence must remain discoverable. |
| Begin a vision, approach documentation, and layer scaffolding. | User message 13 | Produce concrete local drafts and a responsibility scaffold. |

---

## Agent requirements accepted as design inputs

| Requirement | Intended consequence | What still needs evidence |
|---|---|---|
| Discoverable operation meaning | Actors choose operations from declared purpose, inputs, constraints, effects, and outcomes. | An unfamiliar actor can complete a task from the available descriptions. |
| Inspectable context | Actors can identify their document, active definition, location, revision, draft, and execution state. | State survives interruption and handover without silent reinterpretation. |
| Precise editing with drafts | Unfinished documents can be assembled without corrupting unrelated values. | Difficult JSON keys, value types, arrays, and rejected operations preserve the intended data. |
| Efficient machine interaction | Agent calls and human interaction reach equivalent operation semantics. | Batches, direct addressing, discovery, and error handling work in realistic workflows. |
| Recoverable execution | Conflicts, uncertain outcomes, and partial effects remain observable. | Recovery tests distinguish completion from retry and prevent unjustified claims. |
| Understandable composition | A reusable definition exposes its dependencies and relevant guarantees. | Independently authored pieces compose without hidden assumptions. |
| Accountable agent behavior | Agents inspect supplied state, preserve user intent, and verify consequences. | Evaluation detects self-consistent definitions that fail the original task. |
| Portable engineering context | A new actor resumes from durable artifacts. | A fresh actor extends the work without the originating conversation. |

---

## Survey disposition

The [survey skill](https://github.com/apnex/mission-kit/blob/main/skills/survey/SKILL.md) explicitly says: "When direction is already fully specified, do not fabricate a survey."

For the scope of documenting this vision and establishing an initial responsibility scaffold, the recorded discussion supplies the required intent.
The authority reference is user message 13 in the captured discussion, read together with messages 5, 7, and 9 and the accepted requirements in message 12.
No survey questions, stakeholder picks, or time measurements are invented.

This is a scoped fixed-intent bypass, not a statement that all design questions have been answered.
The layer boundaries are implementation proposals, and their open questions remain in the target architecture.
A later design choice that introduces new product intent requires clarification at that point.

---

## Interpretation of the initial scaffold

The scaffold establishes named responsibility directories, their visions, the proposed dependency graph, and the approach for validating later implementation.
It does not claim a stable public API, a completed configuration language, or an operational CLI.

Rust is an implementation preference discussed in the supplied prior conversation.
The scaffold does not select crates, extension mechanisms, a terminal library, or a storage format for durable sessions.
Those choices depend on the concrete contracts and first implementation scope.

The preceding paragraphs retain the initial scaffold boundary.
The subsequent [CLI-001 selection](context/cli-001-approval.json) authorized the [first authoring design](authoring/CLI-001.md), which now supplies bounded engineering contracts and build choices.

The owner's combined request authorizes drafting the vision and beginning the scaffold together.
The [artifact bootstrap methodology](https://github.com/apnex/mission-kit/blob/main/methodology/M8-artifact-bootstrap.md) normally calls for ratifying one artifact type before beginning another.
Here the accompanying documents are explicitly provisional work within that combined request; no separate ratification or implementation approval is asserted.

---

## Mechanics, rationale, and consequence

### Mechanics

Every intent row points to an observed source message.
Implementation inferences remain separate from that source and from any claim of approval.

### Rationale

A new actor needs both the owner's correction and the boundary of what has actually been specified.

### Consequence of violation

Treating the layer proposal as settled intent would convert a reversible scaffold into an architectural commitment the owner did not make.
