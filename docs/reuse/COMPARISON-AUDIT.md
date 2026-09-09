# Authoring comparison design audit

**Verdict: pass-with-guardrails before implementation.**
Scope is the [comparison contract](COMPARISON-CONTRACT.md), under [owner selection](../context/cli-005-comparison-approval.json).
This is an engineering self-audit, not independent certification.
The source observations are the Rust launcher, authoring handler registry and constructor codec, file activation handler, existing continuation transport, and strict task oracle.

## Axiom mapping and tensions

| Axiom | Weight | Application and guardrail |
|---|---|---|
| A3, Sovereign Composition | Load-bearing | Keep comparison policy in trial tooling. Reuse the existing process transport, schema checker, and semantic oracle; do not add a kernel import feature to make the experiment pass. |
| A4, Zero-Loss Knowledge | Load-bearing | Retain original tasks and historical actor records whole, plus all method recipes, external edits, failures, and exact artifacts. |
| A8, Gated Recursive Integrity | Load-bearing | A shared artifact pass cannot replace the stricter continuation verdict. Test wrong-but-valid data and failed edit publication, as well as positive runs. |
| A11, Cognitive Minimalism | Load-bearing | Scripts execute known recipes and count actual bytes. Unknown model/token costs stay null; the previous fresh-agent duration is not a script baseline. |

The tension is outcome comparability versus differing workflow guarantees.
Resolve it with two explicit verdicts and a reported parity gap, preserving the original strict predicate.
An artifact-only result cannot close full workflow economics or justify a new runtime contract.

---

## Layered application and closeout hooks

| Layer | Required check |
|---|---|
| Product | No application source, registered operations, or checkpoint formats change. |
| Trial mechanism | Identical initial artifacts; separate working copies; no overwritten evidence; literal edits publish only after all anchors pass. |
| Task oracle | Shared definition, schema, exact values, labels, exports, and runtime assertions; original staged-continuation requirements remain intact. |
| Measurement | Shared preparation, per-method CLI records, external edits, file volume, and overlapping timers retain their scopes. |
| Documentation | Board and backlog retain incomplete general composition and full-workflow parity; scripts do not become agent-performance evidence. |

Required deltas before implementation: none beyond the contract's explicit import-gap disposition and two-verdict model.
Closeout must cite actual three-method traces, adversarial checks, and the full-continuation result for every method.

---

## Mechanics, rationale, and consequence

### Mechanics

Audit the measured product boundary and near-final experiment before adding trial code.

### Rationale

The evaluator is a future consumer of this design, so acceptance scopes must already be explicit.

### Consequence of violation

A weaker comparator could silently redefine success and make the preferred workflow appear superior without equivalent outcomes.
