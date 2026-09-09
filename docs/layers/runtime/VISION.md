<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Configured runtime vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> Give a loaded CLI definition a consistent, inspectable operational meaning.

---

## Purpose and boundaries

Discovery, input interpretation, constraint application, and invocation need a common interpretation of the active definition.

This scope excludes:

- Rendering terminal keystrokes
- Owning the implementation of domain capabilities
- Changing an active definition implicitly while it is being authored

---

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

- Discovery and invocation refer to the same active operation model.
- Definition changes have explicit activation boundaries.
- The runtime reports what it knows about a result without promoting it to stronger evidence.

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
