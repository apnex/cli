# Component assembly design audit

**Verdict: pass-with-guardrails before implementation.**
Identity: CLI-005 component assembly, selected by the [approval](../context/cli-005-assembly-approval.json).
Design: [assembly contract](CONTRACT.md); independent consumer: [task](acceptance/TASK.md).
Authority: local engineering self-audit, not independent certification or a merged-code gate.
Grounding: existing definition validator, typed mock paths and interpreter, registered composition handlers, checkpoint receipt validation, and the preserved whole-state inspection observed in CLI-005.
Doctrine: workspace standing context and canonical mission-kit A3, A4, A5, A7, A8, and M7, read before this verdict.

## Axiom mapping and layered application

| Principle | Weight | Boundary and enforceable obligation |
|---|---|---|
| A3 composition | Load-bearing | One resolver owns dependency/namespace expansion; the existing interpreter executes its output. Every component state path is scoped, including empty paths. |
| A4 fidelity | Load-bearing | Retain exact source definitions, dependencies, canonical content identities, raw source observations, source task, and historical trial evidence. |
| A5 parity | Load-bearing | One declared operation feeds terminal and machine requests; both inspect the same candidate, tree, dependencies, and invocation labels. |
| A7 recovery | Load-bearing | Stage the entire candidate privately; rejected reads, graphs, and publications cannot leave a partial assembly. Replay and reopen cannot reread missing sources. |
| A8 integrity | Load-bearing | Check the complete expanded artifact against embedded sources on activation and recovery; demonstrate rejection of an applied wrong-scope mutation. |

## Tensions and deltas

A flattened executable definition is convenient but can lose component provenance.
Resolve this by embedding the sources and checking the expansion; derived views cannot silently disagree.
Embedding increases artifact size; retain existing bounds and report limit rejection instead of hiding the duplication.
A dependency denotes required presence and identity, not state-sharing authority or semantic compatibility across arbitrary revisions.
The new artifact format makes that boundary explicit.
No new agent launch or independent-assurance claim is required for the authorized local implementation.
The selected scope supplies sufficient intent; the approval record retains the explicit survey bypass.
No pre-implementation design revision remains outstanding.

## Guardrails and closeout hooks

1. Preserve original component documents and old whole-prototype acceptance cases.
2. Exercise independent state behavior, including root scalar/array replacement and nested contexts.
3. Reject missing/cyclic dependencies and both direct and generated naming collisions before publication.
4. Reject edited executable paths that escape the source component expansion; prove the mutation changed the tested artifact.
5. Verify canonical component digests, exact numeric tokens, source-free transfer/replay, and publication failure/uncertainty.
6. Execute the documented workflow and record exact source/binary identities and test outcomes.
7. Keep connected capabilities, general package resolution, and comparative agent savings outside completion claims.

The subsequent [results](RESULTS.md) map these guardrails to passing application tests, applied mutation rejection, publication-fault observations, and the executed workflow.

## Mechanics, rationale, and consequence

### Mechanics

Interrogate the selected contract against measured runtime boundaries before implementation and carry every guardrail into acceptance.

### Rationale

The existing shared-state observation supplies a concrete consumer for isolated component assembly.

### Consequence of violation

A successful tree rendering could otherwise be mistaken for preserved component behavior or recoverable assembly.
