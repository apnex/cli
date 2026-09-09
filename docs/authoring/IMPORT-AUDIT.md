# Document import design audit

**Verdict: pass-with-guardrails before implementation.**
Scope is the [import contract](IMPORT.md), selected by the [owner's agreement](../context/cli-005-import-approval.json).
This is a local engineering self-audit against the standing doctrine and canonical mission-kit axioms, not independent certification.
Source observations are the registered authoring handlers, exact document parser, bounded file reader, transaction/receipt runtime, and separate CLI activation handler.

## Alignment and tension

| Axiom | Weight | Concrete application |
|---|---|---|
| A3, Sovereign Composition | Load-bearing | Add one authoring operation using existing parsing and persistence; keep schema acceptance and interface activation separate. |
| A4, Zero-Loss Knowledge | Load-bearing | Preserve exact numeric tokens, reject duplicate keys, retain observed source-byte identity, and preserve old checkpoints and evidence. |
| A5, Perceptual Parity | Load-bearing | Both presentations use one declared handler; resulting root context, dirty state, source identity, and changed paths are visible. |
| A7, Resilient Agentic Operations | Load-bearing | Persist the imported draft and receipt together; source removal must not break last-receipt replay or checkpoint reopening. |
| A8, Gated Recursive Integrity | Load-bearing | Test malformed input, typed repair, stale requests, publication failure, and the actual producer/consumer round trip. |
| A11, Cognitive Minimalism | Supporting | Reuse existing deterministic codecs and dispatch rather than adding a parallel import grammar or conversion in the agent. |

The main tension is replacing useful draft content while preserving recoverable session meaning.
Resolve it with an explicitly documented whole-candidate replacement, unchanged accepted baseline, root focus, and the existing diff/discard/commit lifecycle.
The declaration digest changes; old checkpoints remain bound to their original executable/declaration pair.
No compatibility bypass or automatic receipt migration is permitted.

---

## Layered guardrails and closeout

| Boundary | Required check |
|---|---|
| Input | File, UTF-8, duplicate-key, exact-number, scalar, depth, and size rules hold before publication. |
| Authoring | Import changes only candidate and document focus plus the normal revision/receipt; it never accepts or activates implicitly. |
| Persistence | Rejected requests preserve bytes; failure and uncertain publication follow the existing recovery contract. |
| Interface use | A separately imported and activated authored definition retains its expected tree, arguments, labels, and mock behavior. |
| Comparison | Preserve the older failed method, measure the import method, and keep staged-state parity separate from command-only authorship and agent savings. |
| Documentation | Record declaration compatibility and measured outcomes; keep frozen task oracles and historical source records intact. |

Required pre-implementation deltas: none beyond these guardrails.
Closeout must retain failing-before/passing-after or applied-mutation evidence, the separate-session journey, actual comparison results, and source/binary identities.
Publishing, external hooks, arbitrary fragment assembly, and a new runtime mode are outside this increment.
The subsequent [implementation results](IMPORT-RESULTS.md) map these guardrails to actual tests, the executed consumer workflow, and retained failures.

---

## Mechanics, rationale, and consequence

### Mechanics

Audit the measured existing boundaries and explicit acceptance predicate before extending the authoring surface.

### Rationale

The importer has a concrete consumer and can fit the current transaction model without changing the meaning of activation.

### Consequence of violation

A successful file load could otherwise hide changed acceptance, lost draft context, or a different runnable definition.
