<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Constraint interpretation vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> Make declared constraints usable as precise guidance throughout structured authoring.

---

## Purpose and boundaries

A document can be authored as data or applied as a rule system without embedding validation policy inside the general document model.

This scope excludes:

- Owning edit history
- Treating all JSON documents as active schemas
- Executing API operations described by an OpenAPI document

---

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

- Schema-as-data and schema-as-constraint have distinct, explicit uses.
- Validation reports identify the violated rule and affected data.
- Unsupported interpretation cannot be mistaken for complete validation.

---

## Authority

The project owner holds this purpose through the parent vision.
Holding this draft does not approve an API, certify behavior, or grant execution authority.
Changes retain their rationale in the registry and the project record.

---

## Mechanics, rationale, and consequence

### Mechanics

The [responsibility record](contract.md) states the proposed concern, dependencies, and unresolved questions.

### Rationale

This local vision keeps the purpose discoverable from the component's own directory.

### Consequence of violation

An actor that treats a provisional scope as implemented behavior would rely on guarantees this document does not establish.
