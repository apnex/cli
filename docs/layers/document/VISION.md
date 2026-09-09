<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Structured documents vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> Preserve the meaning of structured data through every supported transformation.

---

## Purpose and boundaries

Contextual authoring requires a precise representation of values and locations that is independent of any CLI grammar or domain schema.

This scope excludes:

- Terminal navigation
- JSON Schema evaluation
- Execution of domain operations

---

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

- Unrelated content survives a supported transformation unchanged in meaning.
- Object keys and array indices cannot be silently confused.
- Supported numeric and textual fidelity is explicit.

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
