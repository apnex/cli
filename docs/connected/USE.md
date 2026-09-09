# Use a configured CLI to read a real JSON catalog

The [contract](CONTRACT.md) separates authored requirements, process authority, and historical observations.
This walkthrough constructs the CLI through typed commands, then uses it in independent processes.
The catalog is a real local file; reading it does not probe whether a service is running.
Requires the repository root, Rust/Cargo, a Unix filesystem, and `jq`.
Run the blocks in the same shell.

## Build and author

```sh
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
```

The recipe uses `object`, `array`, and typed scalar constructors without raw JSON containers.
It exports `connected-cli.json` with a connected inspection command, a simulated quota command, and an unbound restart command.

## Grant a catalog and invoke the interface

```sh
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
```

`discover` reports the capability requirement and `granted: true` for this process.
The invocation records the source byte count and digest, exact output text, and its digest.
The target filename lives in the runtime grant; exporting the interface does not transfer that authority.

## Observe a change without changing the definition

```sh
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
```

A fresh invocation observes the new bytes; the authored and accepted CLI definition is unchanged.
The earlier interface export still contains the earlier observation.
Exact replay of the latest persisted machine request returns its saved observation; it is not a refresh.

## Transfer history and restore authority explicitly

```sh
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
```

The saved interface resumed without its import file.
The receiving process supplied a new grant and received a new observation.
Generated files remain under the printed directory; preserve the checkpoint and its matching binary/declaration if you need to resume it later.
No service or background process is installed.

## Mechanics, rationale, and consequence

### Mechanics

Author a portable connected requirement, grant a local target at launch, invoke it, and distinguish historical state from refreshed output after handover.

### Rationale

This makes connected behavior useful while keeping specification reuse independent of local authority.

### Consequence of violation

An exported definition could otherwise appear to carry access, or a stored catalog could be mistaken for a current external observation.
