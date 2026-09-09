set -euo pipefail
. docs/evidence/verb-tree/toolchain-env.sh
cargo build --locked --release --bin cli
assembly_project_root="$PWD"
assembly_demo_dir=$(mktemp -d "$PWD/target/component-cli-XXXXXX")
mkdir "$assembly_demo_dir/services" "$assembly_demo_dir/queues" "$assembly_demo_dir/assembly" "$assembly_demo_dir/consumer" "$assembly_demo_dir/source-archive"
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

printf '%s\n' "$assembly_demo_dir" > "$assembly_project_root/docs/evidence/component-assembly/walkthrough-location.txt"
