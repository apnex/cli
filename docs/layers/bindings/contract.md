<!-- Generated from docs/layers.json; edit that declaration instead. -->
# Behavior bindings responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

Own operation-to-behavior bindings.

---

## Consumes

- Declared operation contracts
- Explicit mock behavior or connected capabilities
- Granted execution authority

---

## Exposes

- Binding availability and compatibility
- Distinguishable unbound, simulated, and real invocation outcomes
- Effect evidence and uncertainty supplied by the connected capability

These are semantic obligations, not implemented type signatures.

---

## Dependencies

- [`definitions`](../definitions/contract.md)

---

## Boundary and evidence

The following concerns are excluded:

- Inferring permission from a newly declared verb
- Fabricating success for an unimplemented operation
- Promising universal rollback or exactly-once external effects

The following observations are required to substantiate this responsibility:

- An unbound operation remains discoverable without implying implemented functionality.
- Mock output is identifiable as simulated.
- Unknown or partial external outcomes remain visible to the caller.

---

## Open questions

- Does the first mock contract expose static examples, simulated state transitions, or both?
- Which extension mechanism satisfies the first connected capability?
- What retry and evidence guarantees can each binding actually provide?

---

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

An interface can be useful before its operations have implementations, while simulated and real effects require distinguishable outcomes.
The architectural anchors for this responsibility are `A3`, `A5`, `A7`, `P5`.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
