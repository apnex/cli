# CLI-003 design audit

## Identity and verdict

Work: [CLI-003](../BACKLOG.md#cli-003), with [owner approval](../context/cli-003-approval.json).
Source: [contract](CONTRACT.md) and [task fixed before implementation](acceptance/TASK.md).
Doctrine: root standing context, axiom definitions identified by [existing source provenance](../evidence/cli-001/knowledge-sources.json), and canonical A0 and [M7](https://raw.githubusercontent.com/apnex/mission-kit/main/methodology/M7-axiom-alignment-audit.md) reread for this extension.
The [authoring audit](../authoring/AXIOM-AUDIT.md) supplies unchanged document and persistence obligations.
Actual runtime, protocol, registrations, terminal compiler, and storage were read before this design and before CLI-003 source changes.

**Verdict: pass-with-guardrails for the selected CLI-003 implementation.**
This is the implementing author's design judgment, not independent certification or application test evidence.

## Axiom mapping and layers

| Axiom | Weight | Application and required observation |
|---|---|---|
| A1, state transparency | Load-bearing | Draft and active definition, simulation, revision, and receipt share one checkpoint; headers distinguish both contexts. |
| A2, specification | Load-bearing | Active data drives discovery, validation, completion, and interpretation; edits do not switch it. |
| A3, composition | Load-bearing | One transaction and storage engine; separate interpretation and simulation modules have real consumers. |
| A4, knowledge | Load-bearing | Exact values, source task, identity, and labeled outcomes survive transfer and restart. |
| A5, parity | Load-bearing | Terminal and machine inputs share invocation semantics and output labels. |
| A6, collaboration | Supporting | Portable state enables later handover; no agent coordinator is claimed. |
| A7, resilience | Supporting autonomous consumers | Replay, revision rejection, locking, and uncertainty cover activation and mocks. |
| A8, integrity | Load-bearing | The task predates the mock model; format validation cannot certify fidelity. |
| A9, failure validation | Supporting, local only | Observed injected faults exercise the local process; no deployment assurance follows. |
| A10, self-evolution | Not materially implicated | Self-authoring does not establish autonomous self-repair. |
| A11, cognitive minimalism | Load-bearing | The kernel owns deterministic typing, serialization, interpretation, and outcome labels. |
| A12, precise context | Load-bearing | Context-local discovery exposes relationships and rejects excessive output explicitly. |
| A13, owner intent | Supporting this work | Approval selects the scope; fixed-intent bypass avoids invented preferences or repeated approval. |
| A14, learning | Supporting | Failures become tests and documented limits; reuse economics remain CLI-005. |

## Tensions and deltas

A separate interface runtime was considered before reading the existing receipt and storage engine.
The selected design extends that engine with versioned state to avoid duplicate transaction behavior.

Configured words can collide with authoring controls.
The `invoke` prefix resolves that collision at the cost of an extra word.
Successful simulated reads consume a revision to retain a durable replayable outcome; they still report simulation-only effects.

Static path checks cannot prove a mock works against every future state.
The interpreter resolves against private evolving state and rejects atomically on failure.
Format 1 binds exact old declarations, so the extension uses an explicit profile and format 2 rather than silent migration.

## Guardrails and closeout hooks

- Keep one publication and replay path for authoring, activation, navigation, and mocks.
- Preserve number spelling through arguments, mock state, output, checkpoint, and transfer.
- Reject missing references, duplicate identities, and unsupported bindings at activation.
- Keep simulation labels outside authored output and verify them on import and reopen.
- Compare complete acknowledged checkpoint bytes on failed replacement and failed multi-step mocks.
- Test changed command and argument names against actual discovery, terminal lowering, and dispatch.
- Establish fault markers before counting injected failure outcomes.
- Run authoring regressions and project checks; distinguish local evidence from independent and remote gates.

No unresolved design deviation requires additional owner approval.
The bounded interpreter does not establish universal CLI coverage.

## Implementation correction

The initial native-scalar request representation failed the exact-number guardrail: a test observed `1e+400` where the task required `1e400`.
The [contract correction](CONTRACT.md#correction-machine-scalar-representation) reuses typed constructors for arguments and exact program text in discovery.
The requirement and independent oracle remain unchanged.
The design verdict remains pass-with-guardrails for this correction before its implementation is accepted; complete fidelity and regression checks still govern closeout.
