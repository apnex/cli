# Programmable CLI vision

**Status: draft expression of recorded project-owner intent.**\
This document states enduring purpose.\
Programmable CLI makes the construction of an interface part of the interface itself: people and agents progressively create the specifications they later inspect, reuse, and operate.\
The [source discussion](docs/context/design-discussion.json) retains the reasoning and corrections behind it.\
The [architecture](docs/ARCHITECTURE.md) interprets that purpose, and the [board](docs/BOARD.md) records selected work; neither changes the intent stated here by reporting an implementation result.

## North star

> Make any project's CLI progressively constructible as a reusable, portable, deterministic definition that people and agents can inspect, operate, and resume.

**Progressively constructible** means an interface takes shape through contextual operations that expose available choices, accept unfinished work, and explain unmet constraints.\
People and agents can construct the definition without manually editing its serialized representation.

**Reusable** means existing definitions and declared capabilities can be assembled and extended across projects.\
Reuse preserves the meaning, requirements, and provenance of the pieces being composed.

**Portable** means the definition and its required context can be carried to another compatible environment or agent without depending on the originating conversation.\
Dependencies and unavailable capabilities remain explicit.

**Deterministic** means the same supported definition, inputs, declared state, and relevant execution conditions have the same interpretation.\
External observations and effects retain their provenance and uncertainty; determinism does not imply control over an external system.

**Resume** means another actor can recover what is defined, what is being authored, what is constrained, what has been observed, and what remains unresolved.\
A resumable definition describes the relevant structure of the target system through entities and explicit relationships.\
It does not stand in for observations of that system.

---

## Purpose

The project makes CLI construction a contextual engineering workflow.\
The resulting interface, discovery, and interaction derive from loaded configuration and assembled capabilities.\
The ambition is that future projects configure or assemble their CLI instead of rebuilding its interaction machinery.

The construction workflow is valuable in its own right.\
A substantial specification can be a successful outcome when the way it is constructed improves correctness, portability, deterministic interpretation, reuse, or continuity of work.\
Specification volume is an optimization dimension, not a disqualifier or a substitute for measuring workflow value.

Structured authoring makes that workflow useful before domain functionality is attached.\
People and agents can construct JSON documents, JSON Schemas, API descriptions, and CLI definitions without managing braces, brackets, or whole-document rewrites.\
They can then use an authored specification to constrain further work.

A CLI definition is also an inspectable account of intended functionality.\
A proposed interface can be explored through declared or simulated behavior before its real implementation exists.\
That experience helps expose missing operations, unclear outcomes, and assumptions that later implementation must resolve.

---

## The composition loop

The same authoring experience can construct the definitions that configure that experience.\
An agent can progressively assemble an interface, inspect its meaning, exercise it, refine it, and make it available to another actor.

The loop retains more than a command vocabulary.\
It carries the constraints, relationships, expectations, decisions, unresolved questions, and relevant observations needed to continue the work.\
The intended result is an inspectable structure that a new agent can reason from without starting the design again.

Self-authorship does not establish correctness against user intent.\
An authored definition remains accountable to the requirements it expresses, and observed behavior remains accountable to independent evidence.

---

## Success dimensions

These dimensions are assessed separately.\
They do not collapse into a single score, and the project claims no measured advantage merely because a definition is valid or a demonstration succeeds.

| Dimension | What succeeding means | Evidence that would challenge success |
|---|---|---|
| Contextual construction | People and agents can discover choices and progressively construct substantial definitions through meaningful operations. | Completion repeatedly requires raw document edits or reconstruction of hidden authoring rules. |
| Semantic fidelity | The intended values, relationships, constraints, and operation meanings survive construction and use. | A valid-looking result changes unrelated content, loses meaning, or disagrees with the original requirements. |
| Portability and determinism | Compatible environments interpret the same definition consistently, with dependencies and external conditions explicit. | Behavior depends on undocumented local state or changes meaning after transfer. |
| Resumption | A fresh actor continues from recorded intent, definitions, work state, and evidence. | The originating conversation or its author's memory is necessary to recover a load-bearing decision. |
| Reuse and reach | Definitions and capabilities compose across increasingly varied projects while retaining understandable contracts. | Each new domain requires rebuilding the common interaction machinery or hides incompatible assumptions. |
| Specification-driven development | A proposed CLI exposes useful implementation expectations before functionality is attached. | A mock implies real effects, or attached behavior is accepted without checking its declared expectations. |
| Shared understanding | Humans and agents can discover equivalent operation meanings and inspect the consequences relevant to their work. | One audience acts on material state or constraints unavailable to the other. |
| Workflow economics | Construction, execution, repair, reuse, and handover improve in a measured and explainable way. | A claimed saving excludes setup or recovery costs, or treats specification length as the sole measure of value. |

---

## Boundaries

| Exclusion | Boundary it preserves |
|---|---|
| Automatic invention of user intent | The person directing the work owns its purpose and acceptable outcomes. |
| Arbitrary functionality created merely by declaring a verb | Domain behavior has an explicit implementation or an explicit simulation. |
| A mock presented as proof of a working system | Declared expectations, simulated results, connected capabilities, and observed effects carry distinct meanings. |
| A mandatory agent runtime or model | People and different agents can use the same declared interaction semantics. |
| A shortest-specification contest | A verbose but valuable construction workflow remains within the purpose of the project. |
| Unbounded authority through self-configuration | Composing an interface does not enlarge the authority granted to its user. |
| Universal coverage asserted from a finite demonstration | Reach is an enduring ambition supported by progressively broader evidence. |

---

## Authority

The project owner who supplied the [recorded discussion](docs/context/design-discussion.json) holds this intent and may change it.\
This draft expresses the owner's stated direction; its exact wording has not been separately ratified.

Holding this document does not authorize an external effect, ratify an architectural choice, or certify implementation behavior.\
The owner's request to document the approach and begin scaffolding supplies the scope of the accompanying work.\
Further changes of direction retain their source and rationale, including corrections to earlier interpretations.

---

## Mechanics, rationale, and consequence

### Mechanics

Use the north star and the separate success dimensions to assess proposed work.\
Trace interpretations to the source discussion and preserve corrections when the intent becomes clearer.

### Rationale

The project is intended to preserve engineering understanding through an executable construction workflow.\
Its own documentation must let a new actor recover that purpose without substituting assumptions about minimal configuration or token savings.

### Consequence of violation

Optimizing the wrong proxy can produce a compact CLI generator while losing the workflow that motivated the project.\
Losing the rationale behind a definition forces the next actor to reconstruct decisions that have already been made.
