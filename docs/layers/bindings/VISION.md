<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Behavior bindings vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> Make the relationship between an intended operation and its available behavior explicit.

---

## Purpose and boundaries

An interface can be useful before its operations have implementations, while simulated and real effects require distinguishable outcomes.

This scope excludes:

- Inferring permission from a newly declared verb
- Fabricating success for an unimplemented operation
- Promising universal rollback or exactly-once external effects

---

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

- An unbound operation remains discoverable without implying implemented functionality.
- Mock output is identifiable as simulated.
- Unknown or partial external outcomes remain visible to the caller.

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
