# Component assembly acceptance task

Construct independent services and queues components entirely through contextual CLI commands.
Each component must remain a complete, inspectable document with its dependency requirements.
The receiving project assembles both components into a worker-platform interface and carries the result to a separate session.

## Required outcomes

| Component | Required behavior |
|---|---|
| Services | Own a service named api with exact quota 1.2300; inspect returns only the service state; quota accepts an exact number and returns it; restart remains explicitly unbound. |
| Queues | Require the services component; own a queue named jobs with paused false; inspect returns only queue state; pause accepts a boolean, changes paused, and returns that boolean. |
| Assembly | Mount services and queues as distinct contexts below root; retain help, signatures, dependency requirements, original component definitions, and simulated labels. |
| Transfer | Save the assembled definition, import into a new session, commit, activate, print tree, and use both components after the original component files are removed. |

The service quota must remain exact when set to 1e400.
Pausing the queue must not change the service inspection result.
Changing service quota must not change queue state.
Both terminal commands and structured machine requests must satisfy this same task.

## Required failures

Missing dependencies, cyclic dependencies, duplicate identities, mount collisions, derived-name collisions, invalid definitions, and unreadable source files must reject assembly without changing checkpoint bytes.
A definition whose executable paths disagree with its embedded components must reject activation and recovery.
A root state read or write inside one component must stay inside that component, including scalar and array state.
Source removal must not break last-receipt replay or committed definition use.
Publication failure and uncertainty must retain the existing transaction contract.

## Mechanics, rationale, and consequence

### Mechanics

Evaluate independently authored pieces before and after assembly and after a separate-session transfer.

### Rationale

Reuse requires preserving each piece's meaning when another piece is added.

### Consequence of violation

A working combined tree could conceal cross-component state access or dependence on the original author's filesystem.
