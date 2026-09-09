# CLI-003 independent task

This task fixes required behavior before implementation of CLI-003.
It is an engineering-selected representative of the owner's [composition intent](../../../VISION.md#the-composition-loop).

Construct a service-catalog interface from an empty authoring document, using contextual commands and typed scalar/container constructors throughout.
The construction trace must not inject an existing serialized definition or be generated from the target document by a test helper.
The interface has root, service, and deployment contexts; services relate explicitly to deployments.
The root explains that the interface is a prototype.
Services expose a simulated read, a quota-setting mock accepting an exact JSON number, and an unbound restart operation.
Initial service state contains name `api`, quota `1.2300`, and enabled `true`.
Deployments expose an example response with status `planned`.

Acceptance predicates:

- Authoring, committing, and saving the definition do not activate it.
- Explicit activation exposes intended contexts, relationships, help, parameters, and bindings.
- Setting quota to `1e400` changes only the simulated quota and retains number spelling; subsequent reads return it.
- A string quota, extra arguments, and unbound restart reject without checkpoint or revision changes.
- Editing a draft leaves the active surface unchanged; invalid activation preserves prior identity, context, simulation, and last outcome.
- Missing references, duplicate operation IDs, unknown formats, and unsupported connected bindings fail activation.
- A valid replacement activates deliberately; reactivating identical definition content preserves simulated state.
- A mock with a successful first step and failing later step publishes neither its earlier step nor a success.
- Export and import into another session preserve definition, context, exact values, simulation labels, and last outcome.
- Reopening after response loss permits replay without repeating the mock; publication uncertainty stops writes until reopen.
- Terminal and machine interactions agree on semantic requests, results, failures, active identities, and files.

Process continuation is tested here.
Independent fresh-agent trials and comparative workflow economics remain CLI-005.
