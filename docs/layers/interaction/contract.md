<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Human and agent interaction responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own interaction presentation.

---

## Consumes

- Runtime discovery and operation results
- Human input or agent requests
- Presentation preferences

---

## Exposes

- Contextual navigation, help, and completion
- Structured machine interaction
- Visible results with accessible detail and provenance

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`runtime`](../runtime/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Reimplementing operation semantics per frontend
- Making a prompt string the only record of context
- Requiring a particular LLM provider

The following observations are required to substantiate this responsibility:

- Equivalent requests through human and machine interfaces have equivalent semantics.
- An actor can discover the context needed for its next operation.
- Concise presentation retains access to the complete relevant evidence.

---

## Open questions

- What grammar handles arbitrary JSON keys and typed values without ambiguity?
- Which machine-call and batch surface best supports the initial authoring workflow?
- What information is supplied by default and what is retrieved on demand?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

Terminal context and machine-readable requests need different presentation while sharing operation meaning and observable state.
The architectural anchors for this responsibility are `A3`, `A5`, `A11`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
