# Component assembly contract

**Status: implemented and verified within the selected local scope.**
The [results](RESULTS.md) retain the application checks and executed walkthrough.
The [approval](../context/cli-005-assembly-approval.json) selects the [acceptance task](acceptance/TASK.md).
These are engineering interpretations of that selected journey, not additional user quotations.

## Operation and lifecycle

The composition profile declares `assemble` with no arguments, state effect, and no batching.
Its input is the complete candidate assembly manifest; component source paths resolve against the process working directory.
All source reads, validation, dependency checks, expansion, and output limits succeed before the candidate is replaced and authoring context returns to root.
The accepted document, attached schema, active interface, and simulated state stay at their prior values.
The result contains the existing typed `changed_paths`; ordinary revision and receipt handling owns publication.
Commit applies an explicitly attached schema.
Activation is explicit.

## Authored documents

| Document | Required shape |
|---|---|
| Component | `format: cli-component-v1`, `requires: array of component identities`, `definition: complete cli-definition-v1 document`. |
| Assembly manifest | `format: cli-assembly-v1`, `id`, `description`, `components: array of {mount, source}`. |
| Assembled definition | `format: cli-definition-v2`, normal executable definition fields, and `assembly.components` keyed by mount with the component snapshot, source spelling, source byte digest, and canonical component digest. |

Unknown fields reject.
A present assembly field must contain a source object; explicit null is rejected.
Nested assembled components are unsupported in this increment.
Each source component identity can appear once.
Dependencies refer to the source definition identity, not its mount name; they require presence and an acyclic graph.
They do not grant access to another component's state or imply connected execution.
There is no dependency downloading or version-range negotiation.

## Expansion and ownership

The global root has no component commands.
A component root becomes its mount context under global root.
A non-root context becomes `mount.local-context`; parent and related references map through the same identity table.
Stable operation identities become `mount.local-operation`.
Command words, help, arguments, and explicit binding classifications retain their meanings.

Simulated state is an object keyed by mount.
Every component state expression and assignment receives the same leading typed key segment.
An empty component path therefore addresses its own state root.
Literal values and argument expressions are unchanged.
Names that collide after expansion reject, including punctuation-induced collisions; identifiers never truncate or silently overwrite.
Existing name, context, command, path, numeric, and document limits apply to the expanded result.

The runtime's global simulated document must have exactly the declared mount keys; each component's value may be any supported JSON root.
The embedded components are the source of the expansion.
Activation and checkpoint/interface recovery recompute and compare executable content against these snapshots.
Editing the generated contexts alone makes the definition inconsistent and rejects.
To refine a component, import its component document, edit through contextual operations, save a new source, and reassemble from the saved manifest.
Keep the manifest before assembly using the existing save operation.

## Portability and identity

The result embeds every component definition and declared dependency.
Source paths and byte digests record the assembly observation and are not authentication or a promise that files still exist.
Canonical component digests bind embedded content; validation checks them without reading source paths.
The same manifest entries in another order produce the same assembled document when source spellings and bytes are unchanged.
Activation and resumption operate on embedded data only.

The assembled format is explicitly new; plain definitions retain their prior serialized shape and behavior.
The composition declaration gains one operation and its digest changes.
Existing checkpoints require their matching prior binary/declaration pair; formats and profile identities are not silently migrated.

## Limits and diagnostics

Assembly accepts 1 through 32 local components.
Each component uses the existing bounded 1 MiB document reader; cumulative source bytes cannot exceed 8 MiB.
The final document, including embedded components and expansion, must fit the existing 1 MiB document limit.
All existing depth, scalar, context, command, and path limits apply.
A limit rejection is an atomic failure, not partial assembly.

| Code | Repair |
|---|---|
| INVALID_CLI_ASSEMBLY | Repair the manifest, snapshot, component shape, or inconsistent expansion. |
| CLI_COMPONENT_SOURCE | Supply a readable bounded regular source file. |
| CLI_COMPONENT_COLLISION | Choose unique mounts and source identities and resolve derived context/operation collisions. |
| CLI_COMPONENT_DEPENDENCY | Supply required identities or remove an invalid, duplicate, self, or cyclic requirement. |
| INVALID_CLI_DEFINITION | Repair an invalid plain component definition or a derived identifier that violates the existing definition rules. |
| LIMIT_EXCEEDED | Reduce the input or resulting artifact to supported bounds. |

## Acceptance and scope

The [task](acceptance/TASK.md) fixes observable behavior independently of the resolver.
Tests must cover both presentations, exact values, root reads and replacements, graph failures, collision failures, source-free transfer/replay, malformed snapshots, and publication faults.
This is a bounded local component resolver.
Connected hooks, shared mutable component state, nested assembly, schema merging, package registries, and general agent savings retain separate consumers and claims.

## Mechanics, rationale, and consequence

### Mechanics

Resolve local component documents once, preserve their snapshots, and derive names and state paths through one expansion function.

### Rationale

The selected worker-platform task needs reusable pieces with inspectable ownership and portable dependencies.

### Consequence of violation

Flattening without retained sources or checked state scopes could silently change behavior while presenting reuse as successful.
