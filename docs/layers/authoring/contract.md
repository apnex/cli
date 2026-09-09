<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Contextual authoring responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own the authoring session.

---

## Consumes

- Document snapshots
- Explicit editing intentions
- A declared acceptance policy

---

## Exposes

- Working context and candidate state
- Atomic edit and batch results
- Pending changes and resumable session state

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`document`](../document/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Implementing schema dialects
- Defining project-specific verbs
- Claiming that a committed document is observed external reality

The following observations are required to substantiate this responsibility:

- A rejected edit has an explicit outcome and does not silently damage the draft.
- Incomplete documents remain inspectable and repairable.
- A new actor can recover the editing context and pending changes.

---

## Open questions

- What does committing mean for local documents and saved definitions?
- Which draft constraints apply immediately and which apply at acceptance?
- How are concurrent changes and stale locations detected?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

Draft state, working location, pending changes, and resumable editing have a distinct lifecycle from document values or domain execution.
The architectural anchors for this responsibility are `A3`, `A4`, `A7`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
