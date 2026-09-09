<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Human and agent interaction vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> Let humans and agents work from equivalent meanings through interaction suited to each.

---

## Purpose and boundaries

Terminal context and machine-readable requests need different presentation while sharing operation meaning and observable state.

This scope excludes:

- Reimplementing operation semantics per frontend
- Making a prompt string the only record of context
- Requiring a particular LLM provider

---

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

- Equivalent requests through human and machine interfaces have equivalent semantics.
- An actor can discover the context needed for its next operation.
- Concise presentation retains access to the complete relevant evidence.

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
