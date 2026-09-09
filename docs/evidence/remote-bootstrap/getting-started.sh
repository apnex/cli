set -eu
getting_started_repo=$PWD
cargo build --locked --release --bin cli
getting_started_cli="$getting_started_repo/target/release/cli"
getting_started_demo=$(mktemp -d)
"$getting_started_cli" --help
"$getting_started_cli" --definition docs/authoring/operations.json \
  --session "$getting_started_demo/document.session.json" --create \
  --intent-file docs/authoring/acceptance/SERVICE-CATALOG.md --commands \
  > "$getting_started_demo/document.events.jsonl" <<'COMMANDS'
set /project string atlas
set /services array
append /services object
edit /services/0
set ./name string api
set ./enabled boolean true
set ./quota number 1.2300
up
top
commit
show
COMMANDS
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$getting_started_demo/document.events.jsonl"
"$getting_started_cli" --definition docs/authoring/operations.json \
  --session "$getting_started_demo/document.session.json" --commands \
  > "$getting_started_demo/reopened.events.jsonl" <<'COMMANDS'
show
COMMANDS
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$getting_started_demo/reopened.events.jsonl"
mkdir "$getting_started_demo/author" "$getting_started_demo/consumer"
(
  cd "$getting_started_demo/author"
  "$getting_started_cli" --definition "$getting_started_repo/docs/authoring/operations.json" \
    --session author.session.json --create \
    --intent-file "$getting_started_repo/docs/connected/acceptance/TASK.md" \
    --compose --commands < "$getting_started_repo/docs/connected/acceptance/catalog.commands" \
    > author.events.jsonl
)
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$getting_started_demo/author/author.events.jsonl"
cp "$getting_started_demo/author/connected-cli.json" "$getting_started_demo/consumer/spec.json"
(
  cd "$getting_started_demo/consumer"
  "$getting_started_cli" run spec.json :tree > tree.txt
  "$getting_started_cli" run spec.json services --help > help.txt
  "$getting_started_cli" run spec.json services quota 12 > quota.json 2> quota.stderr
)
cat "$getting_started_demo/consumer/tree.txt"
cat "$getting_started_demo/consumer/quota.stderr"
test "$(cat "$getting_started_demo/consumer/quota.json")" = '{"quota":12}'
(
  cd "$getting_started_demo/consumer"
  "$getting_started_cli" run --session run.session.json spec.json \
    > run.stdout 2> run.stderr <<'COMMANDS'
services
quota 15
:up
:status
:exit
COMMANDS
  "$getting_started_cli" run --json --session run.session.json - :status > status.json
)
jq -e '.result.last_invocation.output_json_text == "{\"quota\":15}"' "$getting_started_demo/consumer/status.json"
printf 'Your example files are in %s\n' "$getting_started_demo"
