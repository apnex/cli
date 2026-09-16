# Output-view design audit

**Verdict: pass-with-guardrails before implementation.**
Identity: CLI-013; [contract](CONTRACT.md), [selected intent](../context/cli-013-selection.json), and [independent upstream oracle](acceptance/TASK.md).
This is an engineering self-audit under canonical mission-kit M7; it is not independent implementation assurance.
Grounding: the inspected AGP templates/renderer, current exact-value model, strict CLI definitions, assembly derivation, invocation receipts, run delivery, and stored-definition identity checks.

## Applicable mapping and layers

| Anchor | Weight | Design constraint |
|---|---|---|
| A1 state transparency | Load-bearing | Keep the original invocation result and separate presentation diagnostics; rendering a saved result cannot change revisions. |
| A2 specification | Load-bearing | Definitions declare expressions and columns; validation and evaluation consume that same model. |
| A3 composition | Load-bearing | Domain fields and conditions live in authored views; shared Rust code owns traversal, bounded evaluation, formatting, and layout. |
| A4 fidelity | Load-bearing | Preserve exact JSON and original research; freeze upstream display oracles before coding. |
| A8 integrity | Load-bearing | Reject malformed expressions, missing references, incompatible input, and exhausted budgets; exercise actual process outcomes. |
| A9 failure validation | Supporting | Preserve effects across renderer and transport failures; no deployment or live-service claim is made. |
| A14 learning | Supporting | Retain selected consumer, reference behavior, limitations, and tests as reusable project evidence. |

The application is stateful and configurable; it does not coordinate autonomous agents or host an LLM.
A5/A11/A12 inform the consuming agent workflow: typed projections remain available, no table parsing is required, and concise views retain access to complete results.
A6/A7/A10/A13 multi-agent/autonomous mandates are not claimed as application conformance.
The owner's existing direction supplies authority for bounded implementation choices without another survey.

## Tensions and resolution

AGP display strings and jq arithmetic cannot replace this application's exact JSON model.
Keep original data plus typed projections; perform decimal-duration rounding with explicit bounds, and compare rendered cells against AGP separately.
New view fields change definition identity: require v3 and retain existing persistent-session mismatch behavior.
Assembled result paths address the returned document, so only view identities and command references are namespaced.

Rendering happens after execution has succeeded and may have published state.
Its failure must be a presentation outcome, never a fabricated mutation-free invocation rejection.
Saved-result rendering reads the existing outcome and performs no provider call.
Unknown view names are checked before invocation; malformed view definitions reject activation.

No AGP domain identifiers enter shared source.
The test fixture and authored definition own their domain vocabulary.
Resource limits are enforced during traversal and before allocating unbounded table padding.

## Closeout hooks

Recheck frozen-oracle equality, unchanged legacy serialization, precise values, mock/provenance labels, no new revision on re-render, scoped assembly references, empty-PATH execution, landed invalid-input mutations, and all contributor gates.
Source and behavioral review must distinguish measured limits from inferred generality.
No unresolved design delta blocks implementation; the stated non-claims remain part of closeout.

## Implementation disposition

The [measured results](RESULTS.md) satisfy the scoped consumer task and record the validation defect found during implementation.
All eight acceptance observations, both application feature configurations, strict lint, documentation checks, release build, and literal guide replay pass locally.
The original upstream oracle remains unchanged.
This disposition closes the selected engineering work; it does not upgrade this self-audit to independent certification.
