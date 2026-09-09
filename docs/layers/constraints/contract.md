<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Constraint interpretation responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own constraint interpretation.

---

## Consumes

- A document under evaluation
- An explicitly selected schema and dialect
- Reference-resolution inputs

---

## Exposes

- Validation findings with document and schema locations
- Supported constraint semantics
- Contextual guidance with stated completeness limits

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`document`](../document/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Owning edit history
- Treating all JSON documents as active schemas
- Executing API operations described by an OpenAPI document

The following observations are required to substantiate this responsibility:

- Schema-as-data and schema-as-constraint have distinct, explicit uses.
- Validation reports identify the violated rule and affected data.
- Unsupported interpretation cannot be mistaken for complete validation.

---

## Open questions

- Which additional dialects, recursive references, or custom vocabularies does a concrete consumer require beyond local-2020-12-v1?
- What reference packaging and compatibility rules should govern cross-document schema reuse?
- Which additional branch-dependent constraints can offer useful guidance beyond the first structural projection?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

A document can be authored as data or applied as a rule system without embedding validation policy inside the general document model.
The architectural anchors for this responsibility are `A3`, `A5`, `A11`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
