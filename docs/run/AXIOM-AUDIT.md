# Run-mode design audit

**Verdict: pass-with-guardrails before implementation.**
Identity: CLI-008; [approval](../context/cli-008-approval.json), [contract](CONTRACT.md), and [task](acceptance/TASK.md).
Authority: local engineering self-audit under the workspace doctrine and canonical mission-kit W14/M7, already hydrated in this conversation.
Grounding: launcher flags, terminal tokenization and scalar lowering, active-interface binding execution, composition handlers, declaration-owned receipt validation, checkpoint construction/publication, existing renderer, and Reedline completion.

## Mapping and layered application

| Anchor | Weight | Obligation |
|---|---|---|
| A3 composition | Load-bearing | One invocation engine serves authoring and run mode; path qualification must not create a second provider or mock interpreter. |
| A4 fidelity | Load-bearing | Preserve exact argv values, exported definitions, historical outcomes, original provenance, and matching runtime pairs. |
| A5 parity | Load-bearing | The same route projection supplies dispatch, help, tree, and completion. Human output keeps simulation visible while machine output retains provenance. |
| A7 recovery | Load-bearing | A qualified call is one transaction; failed calls preserve navigation and mock state; existing receipt publication owns recovery. |
| A8 integrity | Load-bearing | Reject ambiguous routes and changed persistent definitions rather than silently masking commands or resetting state. Test actual process exits and terminal behavior. |

## Tensions and deltas

The original CLI-003 scope deferred a domain shell; the user's correction now selects it explicitly.
This is an added consumption surface, not evidence that the earlier mock task failed.
An implementation that internally commits `enter` before `invoke` would leave navigation changes after a rejected user command.
Resolve that flaw in the design by qualifying the invocation target within one existing dispatcher request.
Receipt validation must compare the actual qualified target; otherwise source-free reopen would reject valid calls or certify the wrong context.

The authoring declaration remains necessary to the shared kernel, but run users must not locate it or construct a draft before using their spec.
Embed the current declaration and initialize the active interface in one initial checkpoint write.
Fresh default state uses a temporary checkpoint so the proven persistence mechanism remains shared; document normal cleanup and crash leftovers without exposing setup details in the normal flow.

Normal stdout should be usable as JSON; explicit simulated results still need classification.
Put the simulation label on stderr and retain the complete binding envelope under `--json`.
Do not confuse a discarded stderr stream with absence of simulation.
Reserved colon controls avoid hiding valid configured verbs such as `help` or `set`.
Generated context aliases can collide; reject the ambiguous projection before creating state.
No pre-implementation revision remains outstanding.

## Closeout guardrails

1. Prove the old binary fails the direct-run task before implementation.
2. Author and export through existing commands, then execute direct verbs in real one-shot and interactive processes.
3. Verify complete output and exit status, including stdout/stderr separation and exact arguments.
4. Compare run results with existing invocation semantics and reject applied ambiguous or stale definitions.
5. Verify qualified calls do not move saved context and failed calls do not publish partial navigation.
6. Observe publication faults, source-free resumption, and grant-free transferred history.
7. Execute the literal walkthrough and actual terminal completion; preserve original failures and source/binary identities.

## Implementation disposition

The [measured results](RESULTS.md#guardrail-disposition) close these guardrails within the selected local run scope.
The pre-implementation verdict remains a design judgment; application evidence is separately captured in the [verification record](../evidence/run-mode/verification.json).

## Mechanics, rationale, and consequence

### Mechanics

Audit the source-grounded presentation design before implementation and carry each guardrail into acceptance.

### Rationale

A conventional CLI surface must preserve the guarantees already established behind it.

### Consequence of violation

Direct-looking commands could conceal multiple commits, lost argument fidelity, ambiguous routing, or false process success.
