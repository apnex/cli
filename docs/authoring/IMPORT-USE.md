# Author a CLI specification and use it separately

Author a definition, export its JSON, and import it into another session for editing and use.
The `--compose` profile provides activation, discovery, and invocation alongside authoring.
Using an activated interface is the current equivalent of "run" mode; there is no separate `run` command.
The [import contract](IMPORT.md) defines replacement, exact values, constraints, and recovery.

## Build and author

Run these shell commands from the repository root with the prerequisites in the [authoring guide](USE.md):

```sh
cargo build --locked --release --bin cli
import_project_root="$PWD"
import_demo_dir=$(mktemp -d "$PWD/target/import-cli-XXXXXX")
mkdir "$import_demo_dir/author" "$import_demo_dir/consumer"
(
  cd "$import_demo_dir/author"
  "$import_project_root/target/release/cli" \
    --definition "$import_project_root/docs/authoring/operations.json" \
    --session author.session.json --create \
    --intent-file "$import_project_root/docs/composition/acceptance/TASK.md" \
    --commands < "$import_project_root/docs/composition/acceptance/catalog.commands" \
    > author.jsonl
)
```

The existing command recipe constructs the entire service-catalog definition through contextual commands, commits it, and exports `catalog-definition.json`.
It contains no raw JSON container literals.

## Import, accept, and use

Give the separate session the exported definition and its own task:

```sh
cp "$import_demo_dir/author/catalog-definition.json" "$import_demo_dir/consumer/spec.json"
printf 'Use the exported service catalog in a separate session.\n' > "$import_demo_dir/consumer/TASK.txt"
(
  cd "$import_demo_dir/consumer"
  "$import_project_root/target/release/cli" \
    --definition "$import_project_root/docs/authoring/operations.json" \
    --compose --session use.session.json --create --intent-file TASK.txt \
    --commands > use.jsonl <<'COMMANDS'
import spec.json
diff
commit
activate
tree
enter services
invoke inspect
invoke quota 1e400
save imported-definition.json
export-interface interface.json
COMMANDS
)
cmp "$import_demo_dir/author/catalog-definition.json" "$import_demo_dir/consumer/imported-definition.json"
```

`import` replaces the whole candidate and returns authoring focus to root.
`commit` accepts it, applying any explicitly attached schema.
`activate` makes its configured contexts and verbs available.
Invocations in this example are simulated and retain their simulation labels.
The comparison of exported files verifies that using the interface did not rewrite the authored definition.

For use without editing, an existing `--compose` session can directly execute `activate spec.json`.
That selects the interface from the file and leaves the staged documents as they were.
Use `import` when the receiving session also needs to edit or accept that definition.

## Resume and inspect

The checkpoint contains the imported document and active interface; reopening does not require the original source file:

```sh
mv "$import_demo_dir/consumer/spec.json" "$import_demo_dir/consumer/spec.source.json"
(
  cd "$import_demo_dir/consumer"
  "$import_project_root/target/release/cli" \
    --definition "$import_project_root/docs/authoring/operations.json" \
    --compose --session use.session.json --commands > resumed.jsonl <<'COMMANDS'
tree
invoke inspect
COMMANDS
)
```

Inspect operation status and print the configured tree using `jq`:

```sh
jq -s -e 'length > 0 and all(.[]; .event == "session_open" or .status == "ok")' \
  "$import_demo_dir/author/author.jsonl" \
  "$import_demo_dir/consumer/use.jsonl" \
  "$import_demo_dir/consumer/resumed.jsonl"
jq -r 'select(.operation == "tree") | .result.verb_tree_text' "$import_demo_dir/consumer/use.jsonl"
```

An existing checkpoint still requires the declaration bytes to which it was bound.
The import addition changes those bytes; use a new session with exported documents, or retain the prior binary/declaration pair for its old checkpoints.
Checkpoints do not migrate silently.
The [controlled comparison guide](../reuse/COMPARISON-USE.md) also exercises external edits followed by import and continuation.

## Mechanics, rationale, and consequence

### Mechanics

Transfer one exported definition into a new authoring session, explicitly accept and activate it, then resume its configured use.

### Rationale

The definition is portable independently of the session that constructed it.

### Consequence of violation

Conflating import, acceptance, activation, and real execution would conceal what the receiving session actually owns and can do.
