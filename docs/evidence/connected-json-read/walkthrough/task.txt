# Connected service catalog acceptance task

Construct a service-catalog CLI entirely through typed authoring commands.
It must expose `services inspect` as a real read of a locally granted catalog, retain a simulated quota command, and retain an unbound restart command.
Export its definition and use it in another session.

The initial external catalog is `{"name":"api","quota":1.2300,"enabled":true}`.
Its later contents are `{"name":"worker","quota":1e400,"enabled":false}`.
Each fresh inspection must return the corresponding exact values and raw source digest.
Neither read may alter candidate, accepted definition, authoring position, constraints, or mock state.
The mock quota command must continue to affect only mock state.
Restart must remain unbound.

The receiving process must explicitly grant the required logical capability.
Activation, discovery, tree, export, and recovery work without authority; a fresh inspection does not.
Copying the interface export must retain the historical observation and must not copy permission to read its source.
Deleting or replacing a source must not change an exact replay of the last acknowledged request.
Rebinding a capability in a new process changes only future observations.

An assembled component must preserve the same connected requirement while keeping its mock state isolated.
Malformed and oversized data, missing sources, symbolic links, special files, unsupported bindings, stale requests, and missing grants must reject without partial checkpoint changes.
A changed parent path must not redirect an already established grant.
Interrupted publication must recover the original checkpoint or the published observation according to the existing storage boundary.

## Mechanics, rationale, and consequence

### Mechanics

Compare actual file bytes, complete checkpoint state, declared labels, and persisted receipts across independently launched processes.

### Rationale

The task distinguishes real file observation, local simulation, transferable description, and receiving-process authority.

### Consequence of violation

A passing mock or saved observation could otherwise be reported as a successful connected invocation.
