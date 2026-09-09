<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Configured runtime responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own configured operation dispatch.

---

## Consumes

- An explicitly activated CLI definition
- An identified session and operation request
- Authoring, constraint, and binding results

---

## Exposes

- Available operations in the current context
- Definition and state identity for each request
- Structured operation results and resulting context

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`authoring`](../authoring/contract.md)
- [`constraints`](../constraints/contract.md)
- [`definitions`](../definitions/contract.md)
- [`bindings`](../bindings/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Rendering terminal keystrokes
- Owning the implementation of domain capabilities
- Changing an active definition implicitly while it is being authored

The following observations are required to substantiate this responsibility:

- Discovery and invocation refer to the same active operation model.
- Definition changes have explicit activation boundaries.
- The runtime reports what it knows about a result without promoting it to stronger evidence.

---

## Open questions

- What request, result, session, and revision contracts are required?
- What remains available after an invalid activation attempt?
- How do long-running operations and interruptions expose progress and uncertainty?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

Discovery, input interpretation, constraint application, and invocation need a common interpretation of the active definition.
The architectural anchors for this responsibility are `A3`, `A5`, `A7`, `A11`, `P5`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
