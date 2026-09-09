<!-- Generated from docs/layers.json; edit that declaration instead. -->
# CLI definitions responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own the declarative CLI model.

---

## Consumes

- Authored CLI definitions
- Referenced schemas and reusable components
- Declared operation expectations

---

## Exposes

- An inspectable model of contexts and operations
- Explicit references between modeled entities
- Definition diagnostics and dependency requirements

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`document`](../document/contract.md)
- [`constraints`](../constraints/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Implementing arbitrary domain behavior
- Treating a model as proof of a target system's state
- Requiring all modeled relationships to be ownership in a single tree

The following observations are required to substantiate this responsibility:

- The meaning of the interface is recoverable from the definition and its declared dependencies.
- Reusable definitions retain understandable requirements and provenance.
- A CLI definition can itself be constructed through contextual authoring.

---

## Open questions

- What is the smallest expressive definition vocabulary for the first real consumer?
- How do stable entity identities relate to editable JSON locations?
- How are references, compatibility, and unresolved dependencies represented?
- Which OpenAPI concepts map to CLI concepts, and which require explicit choices?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

Commands, contexts, relationships, constraints, and intended outcomes need a common declared meaning that survives reuse and transfer.
The architectural anchors for this responsibility are `A3`, `A4`, `A11`, `P5`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
