#!/usr/bin/env bash
set -euo pipefail
cd /home/apnex/taceng/cli
. docs/evidence/verb-tree/toolchain-env.sh
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
jq -s -e 'length > 0 and all(.[]; .event == "session_open" or .status == "ok")' \
  "$import_demo_dir/author/author.jsonl" \
  "$import_demo_dir/consumer/use.jsonl" \
  "$import_demo_dir/consumer/resumed.jsonl"
jq -r 'select(.operation == "tree") | .result.verb_tree_text' "$import_demo_dir/consumer/use.jsonl"

printf '%s\n' "$import_demo_dir" > docs/evidence/document-import/walkthrough-location.txt
