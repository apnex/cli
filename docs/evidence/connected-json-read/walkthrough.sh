#!/usr/bin/env bash
. docs/evidence/verb-tree/toolchain-env.sh
set -eu
connected_repo=$PWD
cargo build --locked --bin cli
connected_cli="$connected_repo/target/debug/cli"
connected_run=$(mktemp -d "$connected_repo/target/connected-read-XXXXXX")
mkdir "$connected_run/author" "$connected_run/consumer" "$connected_run/recipient" "$connected_run/archive"
cp docs/connected/acceptance/TASK.md "$connected_run/task.txt"
(
  cd "$connected_run/author"
  "$connected_cli" --definition "$connected_repo/docs/authoring/operations.json" \
    --session session.json --create --intent-file ../task.txt --compose --commands \
    < "$connected_repo/docs/connected/acceptance/catalog.commands" > author.jsonl
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$connected_run/author/author.jsonl"
cp "$connected_run/author/connected-cli.json" "$connected_run/consumer/spec.json"
cp docs/connected/acceptance/catalog-initial.json "$connected_run/consumer/catalog.json"
(
  cd "$connected_run/consumer"
  "$connected_cli" --definition "$connected_repo/docs/authoring/operations.json" \
    --session session.json --create --intent-file ../task.txt --compose --commands \
    --grant-json-read services.catalog catalog.json > first.jsonl <<'COMMANDS'
import spec.json
commit
activate
tree
enter services
discover
invoke inspect
export-interface historical.json
COMMANDS
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$connected_run/consumer/first.jsonl"
jq -r 'select(.operation == "tree") | .result.verb_tree_text' "$connected_run/consumer/first.jsonl"
jq -es 'map(select(.operation == "invoke")) | length == 1 and .[0].result.invocation.binding == "connected" and .[0].result.invocation.effect == "external_read" and .[0].result.invocation.output_json_text == "{\"enabled\":true,\"name\":\"api\",\"quota\":1.2300}"' "$connected_run/consumer/first.jsonl"
cp docs/connected/acceptance/catalog-later.json "$connected_run/consumer/catalog.json"
(
  cd "$connected_run/consumer"
  "$connected_cli" --definition "$connected_repo/docs/authoring/operations.json" \
    --session session.json --compose --commands \
    --grant-json-read services.catalog catalog.json > later.jsonl <<'COMMANDS'
invoke inspect
save unchanged-spec.json
COMMANDS
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$connected_run/consumer/later.jsonl"
jq -es 'map(select(.operation == "invoke")) | length == 1 and .[0].result.invocation.output_json_text == "{\"enabled\":false,\"name\":\"worker\",\"quota\":1e400}"' "$connected_run/consumer/later.jsonl"
cmp "$connected_run/consumer/spec.json" "$connected_run/consumer/unchanged-spec.json"
cp "$connected_run/consumer/historical.json" "$connected_run/recipient/historical.json"
mv "$connected_run/consumer/catalog.json" "$connected_run/archive/catalog.json"
mv "$connected_run/consumer/spec.json" "$connected_run/archive/spec.json"
(
  cd "$connected_run/recipient"
  "$connected_cli" --definition "$connected_repo/docs/authoring/operations.json" \
    --session session.json --create --intent-file ../task.txt --compose --commands \
    > without-grant.jsonl <<'COMMANDS'
activate historical.json
discover
invoke inspect
COMMANDS
)
jq -es 'map(select(.operation == "discover"))[0].result | .context.commands.inspect.binding.granted == false and .last_invocation.output_json_text == "{\"enabled\":true,\"name\":\"api\",\"quota\":1.2300}"' "$connected_run/recipient/without-grant.jsonl"
jq -es 'map(select(.status == "error")) | length == 1 and .[0].error.code == "CAPABILITY_NOT_GRANTED"' "$connected_run/recipient/without-grant.jsonl"
mv "$connected_run/recipient/historical.json" "$connected_run/archive/historical.json"
cp docs/connected/acceptance/catalog-later.json "$connected_run/recipient/catalog.json"
(
  cd "$connected_run/recipient"
  "$connected_cli" --definition "$connected_repo/docs/authoring/operations.json" \
    --session session.json --compose --commands \
    --grant-json-read services.catalog catalog.json > regranted.jsonl <<'COMMANDS'
discover
invoke inspect
COMMANDS
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$connected_run/recipient/regranted.jsonl"
jq -es 'map(select(.operation == "invoke")) | length == 1 and .[0].result.invocation.output_json_text == "{\"enabled\":false,\"name\":\"worker\",\"quota\":1e400}"' "$connected_run/recipient/regranted.jsonl"
printf 'Retained connected workflow: %s\n' "$connected_run"
printf "%s\n" "$connected_run" > "$connected_repo/docs/evidence/connected-json-read/walkthrough-location.txt"
