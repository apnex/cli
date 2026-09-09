# Platform mock and verb tree

Construct a platform CLI from an empty document using authoring commands only.
The root has a version verb.
Services expose simulated inspection and replica changes, plus an unbound restart.
Deployments expose a simulated plan and an unbound apply, with a nested releases context for listing and unbound rollback.
Environments expose simulated listing and selection.
The tree must derive context ownership, local command words, required parameters, and binding labels from the activated definition.
Discovery must leave the checkpoint unchanged and retain the current contextual details for machine consumers.
Draft edits must appear in the tree only after successful activation.

This demonstrates local mocked behavior; it does not connect to a deployment platform.
