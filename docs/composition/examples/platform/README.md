# Construct a platform CLI and print its verbs

This example builds a platform mock from an empty document through [authoring commands](platform.commands), then explicitly activates it.
The [task](TASK.md) states its intended behavior.
Services can simulate replica changes, environments can simulate selection, and deployment operations expose their contracts.

## Construct the mock

Run from the repository root with Rust and Cargo available.
The supplied authoring declaration is the bootstrap; the new platform definition is produced by commands.
Each run creates a new directory under the ignored build directory.

Build and construct the mock:
```sh
cargo build --locked --bin cli
cli_project_root="$PWD"
cli_platform_dir="$(mktemp -d "$cli_project_root/target/platform-demo-XXXXXX")"
(
  cd "$cli_platform_dir"
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --compose --session session.json --create \
    --intent-file "$cli_project_root/docs/composition/examples/platform/TASK.md" \
    --commands < "$cli_project_root/docs/composition/examples/platform/platform.commands" \
    > construction.jsonl
)
```

The new directory contains the checkpoint, construction events, and generated `platform-definition.json`.
The command trace supplies scalar values and typed object/array constructors, with no serialized container literals or imported definition.

## Print and explore the tree

Open the constructed session:
```sh
"$cli_project_root/target/debug/cli" \
  --definition "$cli_project_root/docs/authoring/operations.json" \
  --compose --session "$cli_platform_dir/session.json"
```

Enter:
```text
tree
discover
enter services
invoke inspect
invoke scale 4
invoke inspect
enter releases
invoke list
```

`tree` prints the full active verb hierarchy.
`discover` reports current context details.
Contexts end with a slash; each verb includes its required parameter types and its binding label.
Use `enter` with a context ID, then `invoke` with a local command word.
The tree is an ownership view; related-context links remain in the discovery details.
The [expected tree](expected-verb-tree.txt) is a fixed acceptance oracle, separate from the renderer.
Press Ctrl-D to close the session before running the next shell command.

For tree text alone in a shell with `jq`, use the structured result:
```sh
printf 'tree\n' |
  "$cli_project_root/target/debug/cli" \
    --definition "$cli_project_root/docs/authoring/operations.json" \
    --compose --session "$cli_platform_dir/session.json" --commands |
  jq -ejr 'select(.operation == "tree" and .status == "ok") | .result.verb_tree_text'
```

Every tree is derived from the activated definition.
Draft edits appear only after successful activation; `tree` does not advance the revision or change checkpoint bytes.
The [current verification](../../../evidence/tree-command/verification.md) records the standalone command and refreshed demo.
The [initial run](../../../evidence/verb-tree/verification.md) retains the earlier discovery-based presentation.

## Scope

All successful configured verbs are explicitly simulated.
Restart, apply, and rollback remain unbound and reject invocation.
Schema constraints, real deployment effects, and reusable component resolution remain outside this example.
Tree and discovery are separate read-only operations; the tree text is identical in human and machine presentations.
