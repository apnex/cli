<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Structured documents responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own the structured document model.

---

## Consumes

- JSON values
- Unambiguous document locations
- Proposed document transformations

---

## Exposes

- Typed values and document locations
- Snapshots and structural differences
- Explicit transformation results

These are semantic obligations, not implemented type signatures.

---

## Dependencies

No other proposed layer is required by this responsibility.

---

## Boundary and evidence

The following concerns are excluded:

- Terminal navigation
- JSON Schema evaluation
- Execution of domain operations

The following observations are required to substantiate this responsibility:

- Unrelated content survives a supported transformation unchanged in meaning.
- Object keys and array indices cannot be silently confused.
- Supported numeric and textual fidelity is explicit.

---

## Open questions

- Which numeric and serialization fidelity guarantees are supported?
- How are arbitrary keys and array locations represented?
- What persistence contract preserves document revisions?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

Contextual authoring requires a precise representation of values and locations that is independent of any CLI grammar or domain schema.
The architectural anchors for this responsibility are `A3`, `A4`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
