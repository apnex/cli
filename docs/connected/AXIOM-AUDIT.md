# Connected read design audit

**Verdict: pass-with-guardrails before implementation.**
Identity: CLI-006, [selection](../context/cli-006-approval.json), [contract](CONTRACT.md), and [acceptance task](acceptance/TASK.md).
Authority: local engineering self-audit; no independent certification or merged-work claim.
Grounding: read the current binding enum and validator, mock interpreter, composition handler registry, dispatcher revision/replay order, receipt validator, and local storage publication code.
The canonical mission-kit M7 and W14 were read for this design; A3, A4, A5, A7, and A8 supply the project audit anchors.

## Mapping and layered application

| Anchor | Weight | Obligation |
|---|---|---|
| A3 composition | Load-bearing | One provider owns file reads; one invocation path is shared by both presentations and assembled definitions. Definition data cannot become authority. |
| A4 fidelity | Load-bearing | Preserve raw source identity, exact output numbers, historical labels, source-free receipts, and old runtime pairs. |
| A5 parity and context | Load-bearing | Discovery distinguishes declared binding, available grant, and historical observation. Tree and outcome labels cannot imply a running service. |
| A7 recovery | Load-bearing | Check revisions before reads; publish observation and receipt together; replay without another read; retain explicit publication uncertainty. |
| A8 integrity | Load-bearing | Fail closed on unsupported providers and absent authority; pin parent directories; reject inconsistent recovered outcomes and applied mutations. |

## Tensions, deltas, and guardrails

The approval selects a first connected boundary but names no production system.
Resolve that scope gap with a local reference catalog, explicitly recorded as the implementer's selection; do not infer permission to restart services or choose an API.
A generic executable protocol would introduce unearned execution and retry policy; leave it open for a mutating or remote consumer.
The existing regular-file helper checks a path before opening it; that alone cannot establish the new grant's resistance to path replacement.
Use a directory descriptor and a no-follow, nonblocking open for this provider, with regular-file verification and bounded reads.
The existing helper is unchanged; no broader filesystem race fix is claimed.

Authority is transient, while observations must survive transfer.
Keep grants outside serialized state and return historical results on receipt replay even without a new grant.
That is access to already stored data, not authority to refresh it.
File reads and checkpoint writes do not form one external transaction; failed checkpoint publication can follow a completed read.
Keep that distinction in the contract and fault tests.
No pre-implementation design delta remains outstanding.

## Closeout hooks

1. Execute the independent task and exact definition oracle through both presentations.
2. Measure the target bytes and unchanged local documents, including precise numbers and payloads that resemble effect labels.
3. Test absent authority, unsupported bindings, bad data, limits, and stale requests before any successful replacement.
4. Rename the granted parent and substitute another path; observe the original directory's file.
5. Transfer assembled and plain interfaces without grants; replay persisted observations without the source.
6. Apply inconsistent outcome mutations and observe rejection; do not claim authenticity.
7. Exercise publication failure and uncertainty with confirmed fault markers; execute the literal walkthrough and record its limits.

The subsequent [results](RESULTS.md) map these guardrails to passing application tests, applied input mutations, confirmed publication faults, and the executed walkthrough.

## Mechanics, rationale, and consequence

### Mechanics

Audit the source-grounded design before building and carry every guardrail into behavioral validation.

### Rationale

Connected observations require more precise authority and recovery statements than simulation alone.

### Consequence of violation

An inherited local atomicity claim could overstate what occurred outside the checkpoint.
