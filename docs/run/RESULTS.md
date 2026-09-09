# Direct run results

**Status: complete within the selected local CLI-008 scope.**
The [approval](../context/cli-008-approval.json), [contract](CONTRACT.md), [audit](AXIOM-AUDIT.md), and [independent task](acceptance/TASK.md) define CLI-008.
Measurements here are local executor evidence on Linux; no independent agent evaluation or deployment is claimed.

The full instrumented application suite passed 87 tests; the normal build passed 70, including nine run tests without fault instrumentation.
The [verification record](../evidence/run-mode/verification.json) identifies the commands, captured logs, source and binary hashes, documentation checks, and retained failures.

## Observed behavior

`cli run spec.json services quota 1e400` returns `{"quota":1e400}` and labels the result `[simulated]` on stderr.
A granted `services inspect` returns real local catalog JSON; an unbound `services restart` exits 1.
No consuming command requires `invoke`, an operations declaration file, or authoring-session setup.
Plain definitions, interface exports, and assembled definitions use the existing source decoder.

The focused suite passed 10 instrumented tests, including a real terminal driven through a pseudoterminal by the Linux `script` utility.
Its [terminal transcript](../evidence/run-mode/targeted-final/run-terminal.raw) records Tab completion, accepting the suggestion, entering the services context, invoking the quota mock, returning to root, and exiting successfully.
The resulting saved checkpoint has three acknowledged transitions and the exact mock result.
Other tests assert command and boolean completion values and replacement spans.

The [literal walkthrough](../evidence/run-mode/walkthrough.log) passed all its shell assertions.
It constructs the spec through 57 typed commands with zero raw JSON container lines, then exercises direct calls, help, tree output, optional persistence, source-free reopen, and portable history.
The original generated directory is retained at the [recorded location](../evidence/run-mode/walkthrough-location.txt); its archived copy compared equal.

## Guardrail disposition

| Guardrail | Measurement |
|---|---|
| Existing failure before implementation | [Baseline](../evidence/run-mode/failing-before.log) compiled the direct-run test and failed because the previous launcher rejected `run`. |
| Shared invocation meaning | Command-authored bare and interface exports produce the same complete invocation outcome as the canonical qualified authoring request. |
| Literal arguments and exact output | OS argv and interactive quoting preserve whitespace, empty strings, quotes, Unicode, control characters, and shell-like text; numeric spelling survives as `1e400` and `1.2300`. |
| Generated surface parity | Help/tree/completion share route words; assembled `primary services echo` resolves the stored `primary.services` context and keeps component state separate. |
| Normal command meanings | Configured `set`, `help`, `tree`, `exit`, and `invoke` remain callable. Colon controls are explicit. |
| Atomic invocation | Qualified calls leave navigation unchanged and increment revision once. Missing, extra, mistyped, unknown, and unbound calls preserve complete persistent checkpoint bytes. |
| Persistence and transfer | Matching specs retain state, changed specs reject without rewriting the checkpoint, and source-free reopen and interface exports preserve historical outcomes. Grants do not transfer. |
| Rejected surfaces | Applied command/context collisions and invalid parent/signature definitions fail before checkpoint creation. |
| Failure and uncertainty | Observed faults before checkpoint write, after rename, and before response delivery produce no success payload. Recovery distinguishes the unchanged checkpoint from a durable new receipt. |
| Process status | Usage/argument errors exit 2; provider, grant, unbound, and publication errors exit 1. Piped input retains an earlier failure even after a later successful command. |

The [focused log](../evidence/run-mode/targeted-final.log) retains test names, three applied definition rejection markers, and three observed publication fault markers.
The temporary run storage and persistent run storage both use the existing transactional dispatcher and receipt publisher.

## Corrections retained

The [first implementation build](../evidence/run-mode/first-implementation.log) failed on a conditional directory-creation expression; no behavioral pass is inferred from that compile attempt.
The [first expanded test run](../evidence/run-mode/targeted-first.log) found that status requests incorrectly carried `expected_revision`, which the read contract rejects.
The lowering now derives revision requirements from operation effect metadata.
The existing authoring prompt also lost its unconditional `(mock)` suffix, which was misleading for a context containing connected reads; runtime outcomes retain their actual binding labels.
Final presentation checks require invalid scalar arguments to point to run-mode `:help`, and a full output destination to return delivery failure with exit 1 even when printing launcher help.

The first terminal driver sent keys while the editor queried cursor position.
The [second attempt](../evidence/run-mode/targeted-second.log) exposed the next driver error: Tab opens the completion menu, and Enter accepts its suggestion before another Enter submits the command.
The [third attempt](../evidence/run-mode/terminal-third.log) recorded successful interaction but killed the `script` wrapper during its exit race.
The final driver waits for the displayed stages and bounded process completion; those earlier transcripts remain failures, not passing terminal evidence.

## Limits

Arguments remain ordered required string, number, and boolean scalars from the current CLI definition format.
No optional or named parameters, automatic OpenAPI mapping, executable hooks, or new providers are introduced.
The connected provider is still only a local JSON file reader; an observation is not a service-health probe.
Default runs use fresh temporary checkpoints removed on normal exit; forced termination can leave them behind.
Persistent checkpoints require their matching binary/declaration pair, and the [previous pair](../evidence/run-mode/before/runtime-location.txt) was preserved before changing invocation metadata.
Interface exports transfer definitions and history across that boundary without rewriting old receipts.
A new shell command is a fresh invocation; exact latest-request replay remains a canonical request-protocol feature.
A delivery failure can follow durable execution, so persistent state must be inspected before retrying.
These observations do not measure comparative agent savings or certify every possible future CLI surface.

## Mechanics, rationale, and consequence

### Mechanics

Project the validated definition into direct routes and lower calls to the existing atomic invocation handler.

### Rationale

An exported specification becomes independently usable while retaining construction, simulation, observation, and recovery semantics.

### Consequence of violation

The apparent CLI could require hidden authoring steps or return success while its state, output, and saved receipt disagree.
