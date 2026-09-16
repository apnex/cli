# Native AGP output-view acceptance

Rebuild AGP's `connections.list` and `routes.list` views through the configurable Rust CLI.
Use the unchanged fixture documents and frozen expected display rows/tables in this directory, captured before implementation from AGP revision `da1a03a370f522bfb2a3afc66fb781b295cb6af7`.
The [oracle manifest](../../evidence/output-views/agp-oracle.sha256) identifies those bytes.

1. Construct the command definitions, view definitions, expressions, and column formats using typed contextual authoring commands; no manually edited JSON definition is accepted as workflow evidence.
2. Export the constructed definition, import it into a separate authoring session, and run the transferred specification's ordinary verbs.
3. Both nonempty and empty connection/route results match the frozen AGP tables and display rows. Pending identity, hold timers, uptime beyond a day, selected-route membership, local/session next hops, booleans, and hostile strings retain the reference behavior.
4. Native rendering succeeds with `PATH` pointing at an empty directory. The delivered feature and consumer walkthrough require no Bash, jq, or column executable.
5. Original JSON results and exact number tokens remain accessible. Table preferences cannot change a binding, grant authority, or alter a stored invocation result.
6. A recorded result can be rendered after removing its source and without new grants, state revisions, or invocation. Simulated results remain labeled.
7. Invalid expressions, dangling view references, incompatible input, work/output limits, and output failure are explicit. A rendering failure after successful execution must not claim the operation was rejected or invite re-execution as the recovery step.
8. Older definitions retain their serialized content and output behavior. Assembling components scopes view identities without rewriting result-document paths.

Reference acquisition used AGP's existing Bash/jq implementation, with Node tests only as research evidence.
Those are not runtime dependencies of this implementation or of the Rust acceptance suite.
No live AGP management endpoint, network provider, full jq interpreter, or measured agent-effort benefit is part of this task.
