set -eu
run_repo=$PWD
cargo build --locked --bin cli
run_cli="$run_repo/target/debug/cli"
run_demo=$(mktemp -d "$run_repo/target/run-mode-XXXXXX")
mkdir "$run_demo/author" "$run_demo/consumer" "$run_demo/archive"
(
  cd "$run_demo/author"
  "$run_cli" --definition "$run_repo/docs/authoring/operations.json" \
    --session author.json --create --intent-file "$run_repo/docs/connected/acceptance/TASK.md" \
    --compose --commands < "$run_repo/docs/connected/acceptance/catalog.commands" > author.jsonl
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$run_demo/author/author.jsonl"
cp "$run_demo/author/connected-cli.json" "$run_demo/consumer/spec.json"
cp docs/connected/acceptance/catalog-initial.json "$run_demo/consumer/catalog.json"
(
  cd "$run_demo/consumer"
  "$run_cli" run spec.json services --help > help.txt
  "$run_cli" run spec.json :tree > tree.txt
  "$run_cli" run spec.json services quota 1e400 > mock.json 2> mock.stderr
  "$run_cli" run --grant-json-read services.catalog catalog.json \
    spec.json services inspect > observed.json
  if "$run_cli" run spec.json services restart > unbound.stdout 2> unbound.stderr; then
    exit 1
  else
    test "$?" -eq 1
  fi
)
cat "$run_demo/consumer/tree.txt"
test "$(cat "$run_demo/consumer/mock.json")" = '{"quota":1e400}'
rg -q '^\[simulated\]' "$run_demo/consumer/mock.stderr"
test "$(cat "$run_demo/consumer/observed.json")" = '{"enabled":true,"name":"api","quota":1.2300}'
test ! -s "$run_demo/consumer/unbound.stdout"
rg -q UNBOUND_OPERATION "$run_demo/consumer/unbound.stderr"
(
  cd "$run_demo/consumer"
  "$run_cli" run --session run.json spec.json > commands.stdout 2> commands.stderr <<'COMMANDS'
services
quota 12
:up
services quota 1.2300
:status
:export portable.json
:exit
COMMANDS
  "$run_cli" run --json --session run.json spec.json :status > reopened.json
)
jq -e '.result.last_invocation.output_json_text == "{\"quota\":1.2300}" and .session.active_interface.context == "root" and .session.revision == "4"' "$run_demo/consumer/reopened.json"
mv "$run_demo/consumer/spec.json" "$run_demo/archive/spec.json"
(
  cd "$run_demo/consumer"
  "$run_cli" run --json --session run.json - :status > source-free.json
  "$run_cli" run --json portable.json :status > transferred.json
)
jq -e '.result.last_invocation.output_json_text == "{\"quota\":1.2300}"' "$run_demo/consumer/source-free.json"
jq -e '.result.last_invocation.output_json_text == "{\"quota\":1.2300}"' "$run_demo/consumer/transferred.json"
printf 'Retained run workflow: %s\n' "$run_demo"
