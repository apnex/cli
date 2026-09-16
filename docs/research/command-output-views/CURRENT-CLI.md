# Current configurable CLI output

**Tier 2: source inspection, 2026-09-16.**

Inspected `apnex/cli` at `543de9bb666b6ab4be45b9b069bf855d3771de80`, with a clean worktree before research documentation was added.
This record describes source and current contracts; the Rust behavior suite was not rerun for this documentation task.

| Surface | Measured current contract or mechanism | Source at the inspected revision |
|---|---|---|
| Document values | `DocumentValue` has object, array, string, number, boolean, and null variants. Numbers retain their validated JSON token in `RawValue`; equality and serialization use that token. | [Document values](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/document_value.rs#L19) |
| Command definitions | A `CliCommand` contains `id`, `help`, `parameters`, and `binding`. It has no result-view declaration. Unknown fields reject decoding. | [Definition types](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_definition.rs#L194) |
| Mock output | Output expressions select a literal, an argument, or a simulated-state path. These are binding expressions, with no table projection or general query vocabulary. | [Mock expressions](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_definition.rs#L28) |
| Invocation outcomes | Runtime-owned metadata surrounds exact `output_json_text`: definition digest, operation ID, context, binding, effect, simulated-step count, and optional connected observation. | [Outcome](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_interface.rs#L25), [invocation](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_interface.rs#L289) |
| Plain run output | Successful invocations write the exact result JSON text and a newline. Simulated invocations add a label to stderr. Help, tree, and other explicit controls use their existing presentations. | [Response delivery](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_run_frontend.rs#L268) |
| Structured run output | `--json` writes the full structured response, including runtime metadata. It does not mean only the invocation payload. | Response delivery; [run contract](../../run/CONTRACT.md#output-and-exit-status) |
| Routing | Launcher options precede the domain command path. Contextual runtime controls have a `:` prefix so configured words remain usable as domain verbs. | [Launcher](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_run_launcher.rs#L19), [use guide](../../run/USE.md) |
| Definition compatibility | Formats and known fields are validated. A persistent run rejects a different supplied definition rather than resetting saved state. | [Definition validation](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_definition.rs#L299), [run session](https://github.com/apnex/cli/blob/543de9bb666b6ab4be45b9b069bf855d3771de80/src/cli_run_session.rs#L80) |
| Delivery boundary | Successful effects can be published before response delivery. Delivery failures must not masquerade as rejected edits. | Response delivery; run contract |

## Declared responsibility boundaries

These are provisional architectural responsibilities, not a claim that a reusable output-view API exists:

- [Definitions](../../layers/definitions/contract.md): the inspectable declarative CLI model, reusable components, requirements, and provenance.
- [Runtime](../../layers/runtime/contract.md): operation dispatch and structured results.
- [Interaction](../../layers/interaction/contract.md): visible presentation, machine interaction, and access to complete relevant evidence.

The inspected command model and delivery path do not implement configurable table views.
