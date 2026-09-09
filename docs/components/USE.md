# Assemble reusable CLI components

Construct two independent component documents, assemble them, and use the resulting CLI in another session.
The [contract](CONTRACT.md) describes namespaces, dependencies, exact values, and source-free recovery.
This walkthrough uses local mocks and an explicitly unbound restart command.

## Prerequisites and build

Run from the repository root with Rust, Cargo, a C linker, Bash, and jq.
The [authoring guide](../authoring/USE.md) covers the general CLI prerequisites.

Build the normal application and create disposable working directories:

```sh
cargo build --locked --release --bin cli
assembly_project_root="$PWD"
assembly_demo_dir=$(mktemp -d "$PWD/target/component-cli-XXXXXX")
mkdir "$assembly_demo_dir/services" "$assembly_demo_dir/queues" "$assembly_demo_dir/assembly" "$assembly_demo_dir/consumer" "$assembly_demo_dir/source-archive"
```

## Author the components

Each recipe contains only typed scalar/container construction and contextual CLI operations.
The files are readable command examples; neither inserts a serialized JSON container.

Execute each recipe in its own authoring session:

```sh
for assembly_component in services queues; do
  (
    cd "$assembly_demo_dir/$assembly_component"
    "$assembly_project_root/target/release/cli" \
      --definition "$assembly_project_root/docs/authoring/operations.json" \
      --session author.session.json --create \
      --intent-file "$assembly_project_root/docs/components/acceptance/TASK.md" \
      --commands < "$assembly_project_root/docs/components/acceptance/$assembly_component.commands" \
      > author.jsonl
  )
  cp "$assembly_demo_dir/$assembly_component/$assembly_component.component.json" "$assembly_demo_dir/assembly/"
done
```

The [services recipe](acceptance/services.commands) defines inspection, exact quota updates, and an unbound restart.
The [queues recipe](acceptance/queues.commands) declares its services dependency and defines inspection and pause.
Component paths are local: an empty state path means that component's whole state.

## Assemble and export

The [assembly recipe](acceptance/assembly.commands) constructs and saves the manifest through the CLI.
Its mount names determine the resulting namespaces.
The final three commands replace the manifest draft with its assembled definition, accept it, and export it.

Run the manifest recipe and assembly in one composition session:

```sh
cp "$assembly_project_root/docs/components/acceptance/assembly.commands" "$assembly_demo_dir/assembly-input.commands"
printf 'assemble\ncommit\nsave worker-platform.json\n' >> "$assembly_demo_dir/assembly-input.commands"
(
  cd "$assembly_demo_dir/assembly"
  "$assembly_project_root/target/release/cli" \
    --definition "$assembly_project_root/docs/authoring/operations.json" \
    --compose --constraints --session assembly.session.json --create \
    --intent-file "$assembly_project_root/docs/components/acceptance/TASK.md" \
    --commands < "$assembly_demo_dir/assembly-input.commands" > assembly.jsonl
)
cp "$assembly_demo_dir/assembly/worker-platform.json" "$assembly_demo_dir/consumer/spec.json"
cp "$assembly_project_root/docs/components/acceptance/TASK.md" "$assembly_demo_dir/consumer/TASK.txt"
mv "$assembly_demo_dir/assembly/services.component.json" "$assembly_demo_dir/assembly/queues.component.json" "$assembly_demo_dir/source-archive/"
```

The exported definition embeds component snapshots and requirements.
Its recorded source paths now have no matching files in the assembly directory, and the consumer receives no component source files.
The saved manifest remains available for later reassembly with revised sources.

## Import and use separately

Import the specification, accept it, activate it, and exercise each component:

```sh
(
  cd "$assembly_demo_dir/consumer"
  "$assembly_project_root/target/release/cli" \
    --definition "$assembly_project_root/docs/authoring/operations.json" \
    --compose --constraints --session use.session.json --create --intent-file TASK.txt \
    --commands > use.jsonl <<'COMMANDS'
import spec.json
commit
activate
tree
enter services
invoke inspect
enter queues
invoke pause true
invoke inspect
enter services
invoke inspect
invoke quota 1e400
export-interface interface.json
save same-definition.json
COMMANDS
)
cmp "$assembly_demo_dir/consumer/spec.json" "$assembly_demo_dir/consumer/same-definition.json"
jq -r 'select(.operation == "tree") | .result.verb_tree_text' "$assembly_demo_dir/consumer/use.jsonl"
jq -s -e '[.[] | select(.operation == "invoke" and .result.invocation.operation_id == "services.inspect") | .result.invocation.output_json_text] as $reads | ($reads | length) == 2 and $reads[0] == $reads[1]' "$assembly_demo_dir/consumer/use.jsonl"
```

The two service reads must match even though the queue changed between them.
The quota invocation uses the exact number token 1e400.
The exported interface includes the updated simulated state; the saved definition retains its authored initial state.
The tree marks the restart command unbound.

## Resume without source files

Move the imported spec aside, then reopen the consumer checkpoint:

```sh
mv "$assembly_demo_dir/consumer/spec.json" "$assembly_demo_dir/consumer/spec.source.json"
(
  cd "$assembly_demo_dir/consumer"
  "$assembly_project_root/target/release/cli" \
    --definition "$assembly_project_root/docs/authoring/operations.json" \
    --compose --constraints --session use.session.json --commands > resumed.jsonl <<'COMMANDS'
invoke inspect
enter queues
invoke inspect
COMMANDS
)
jq -s -e 'length > 0 and all(.[]; .event == "session_open" or .status == "ok")' \
  "$assembly_demo_dir/services/author.jsonl" "$assembly_demo_dir/queues/author.jsonl" \
  "$assembly_demo_dir/assembly/assembly.jsonl" "$assembly_demo_dir/consumer/use.jsonl" \
  "$assembly_demo_dir/consumer/resumed.jsonl"
jq -s -e 'any(.[]; .result.invocation.operation_id == "services.inspect" and .result.invocation.output_json_text == "{\"name\":\"api\",\"quota\":1e400}") and any(.[]; .result.invocation.operation_id == "queues.inspect" and .result.invocation.output_json_text == "{\"name\":\"jobs\",\"paused\":true}")' "$assembly_demo_dir/consumer/resumed.jsonl"
```

To revise a component, reopen its authoring session, use contextual edits, and save a new component file.
Use the saved assembly manifest with the desired source files, then assemble again.
The generated executable fields are checked against their embedded components; edit the sources rather than creating a disagreement between those representations.

An older composition checkpoint still needs its matching binary and declaration.
Adding the assemble operation changes the composition declaration digest.
Preserve older runtime pairs before cleaning build output.
Save the demonstration directory if its sessions and exports are useful; it contains no installed service.

## Mechanics, rationale, and consequence

### Mechanics

Author independent pieces through commands, assemble once, compare component behavior, and transfer only the resulting definition.

### Rationale

Reuse preserves each piece's meaning while allowing a new project to combine them.

### Consequence of violation

A combined tree could work only on the original machine or expose one component's state through another component's commands.
