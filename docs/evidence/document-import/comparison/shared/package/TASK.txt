# Worker-platform continuation task

This task is fixed before its completion trace and acceptance implementation.
It is an engineering-selected second consumer of the service-catalog prototype.
The supplied original catalog task remains the authority for reused behavior.

## Required extension

Extend the unfinished definition into a worker-platform prototype through contextual CLI operations.
Do not edit JSON documents, checkpoints, or exported bundles directly.
Preserve the complete `services` and `deployments` context definitions and the root `about` command from the original catalog.
Their IDs, relationships, help, signatures, bindings, exact numbers, and unbound restart requirement remain unchanged.

| Part | Required result |
|---|---|
| Definition | ID `worker-platform`; description `Worker platform prototype; all behavior is simulated or unbound.` |
| Root | Help `Explore the worker platform prototype.`; original parent, relationships, and commands unchanged. |
| Queue context | ID `queues`, parent `root`, relationship to `services`, help `Inspect and simulate a job queue.` |
| Declared state | Original name `api`, quota token `1.2300`, enabled `true`; new `queue` object with name `ingest`, depth integer `0`, paused boolean `false`. |
| Queue `inspect` | ID `queue.inspect`; no parameters; simulated read returning only the queue object; help `Read simulated queue state.` |
| Queue `pause` | ID `queue.pause`; required boolean parameter `paused`, help `New simulated pause state.`; one simulated write to `queue.paused`, then return the queue object; command help `Set the simulated queue pause state.` |
| Queue `drain` | ID `queue.drain`; no parameters; unbound reason `No queue-drain capability has been attached.`; help `Drain the real queue when an implementation is attached.` |
| Other changes | None. No extra contexts, commands, parameters, or state members. |

## Continuation obligations

- Inspect the current draft, authoring location, accepted baseline, active interface, last simulated outcome, and attached schema before editing.
- Attempt the abandoned request once through the machine interface; observe its stale revision rejection and preserve current work. Do not refresh and execute that abandoned deletion.
- Attempt to open the session with the supplied incompatible operation declaration; observe the dependency mismatch without changing the checkpoint, then use the supplied compatible declaration.
- Keep the attached policy unchanged. It checks definition structure and queue scalar types; kernel activation checks the full command graph and binding semantics.
- Attempt a commit of the supplied invalid draft, inspect its field error, and repair through typed operations. Preserve the accepted baseline on rejection.
- Complete and commit the definition, activate it explicitly, and inspect its verb tree.
- Exercise service quota with exact number `1e400`, then queue pause with boolean `true`. The final simulated quota is `1e400`, queue depth is `0`, and queue pause state is `true`.
- Leave interface context `queues` and authoring context `/contexts/queues/commands`. Activation initializes a changed definition from its declared mock state; it does not migrate earlier simulated state.
- Save the completed definition as `completed.definition.json` and export the active interface as `completed.interface.json` after the final state changes.
- Retain all requests, results, failures, repairs, and any human intervention. Report any requirement that could not be completed.

## Acceptance boundary

Correctness is evaluated against this task and the original catalog requirements, independently of the authored schema and mock outputs.
The evaluator checks preserved definitions, exact declared and simulated values, labels, whole-operation rejection, both exports, and continuation state.
It also probes string input to `pause`, unbound `drain` and `restart`, and service quota after transfer using a separate evaluation copy.
All behavior remains local and simulated or unbound.
There is no queue, service, restart, drain, network interaction, or real deployment to certify.

## Mechanics, rationale, and consequence

### Mechanics

Use the supplied CLI to extend the checkpoint and retain an inspectable command trace.

### Rationale

This task requires reuse, repair, and continuation together; constructing a new queue interface alone does not satisfy it.

### Consequence of violation

A plausible new interface can pass its own schema while silently changing the reused service contract or losing the unfinished work.
