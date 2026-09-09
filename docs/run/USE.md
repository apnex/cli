# Run an authored CLI directly

Run mode loads a CLI definition, assembled definition, or interface export and exposes its configured verbs.
It starts with fresh state unless you supply `--session`.
The [contract](CONTRACT.md) defines routing, controls, persistence, and output.
This walkthrough requires the repository root, Rust/Cargo, a Unix filesystem, and `jq`; run its shell blocks in the same shell.

## Build and construct a specification

```sh
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
```

The recipe constructs the exported CLI through typed authoring commands without raw JSON containers.
The consumer needs the resulting spec and the executable.

## Use the configured verbs

```sh
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
```

Successful invocation output is exact JSON on stdout.
Mocks announce `[simulated]` on stderr; an unbound operation exits 1.
A malformed command or argument exits 2.
Use `--json` before the command path to receive the full response, including binding and observation metadata; structured errors go to stderr.

## Navigate and retain state

```sh
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
```

A context word on its own enters that context in a command stream or interactive shell.
A qualified command such as `services quota 1.2300` leaves the current navigation context unchanged.
One-shot command paths always start at root.
A persistent run resumes its prior state and rejects a different definition without resetting it.
Use `-` with `--session` to resume without the source file.
`:export` creates a portable interface with state and history; receiving processes supply their own grants for fresh connected reads.

For an interactive terminal, omit the command and input redirection: `cli run --session run.json spec.json`.
Tab completes local contexts, commands, and boolean values; the prompt shows the selected context.
`:help`, `:tree`, `:up`, `:top`, `:status`, `:export <file>`, and `:exit` are shell controls.
Configured commands named `help`, `tree`, `set`, or `exit` retain their domain meaning.
Arguments are required ordered scalars from the spec; optional arguments and named flags are outside the current format.
OS argv preserves each argument literally; interactive whitespace-containing strings use JSON string quoting.
Use `command -- --help` for a literal help argument and `spec.json -- --json ...` for a configured command named `--json`.

## Verify and preserve

Run `cargo test --locked --all-targets`, `cargo test --locked --all-features --all-targets`, and `cargo fmt --all -- --check` from the repository root.
The run acceptance suite includes a Linux terminal test using the `script` utility from util-linux, which must be installed.
Keep generated checkpoints with their matching binary and composition declarations; changing embedded declaration bytes changes the checkpoint identity.
Portable interface exports transfer between kernel declaration versions without migrating old receipts.
Fresh runs remove their temporary checkpoint directory on normal exit; persistent checkpoints and exports remain at the paths you selected.
A delivery failure can follow durable execution: inspect the saved receipt before retrying, because another shell command is a new invocation.

## Mechanics, rationale, and consequence

### Mechanics

Construct a spec, consume it through direct verbs, and independently reopen or transfer its acknowledged state.

### Rationale

The reusable artifact becomes a usable CLI without requiring its operator to enter the authoring workspace.

### Consequence of violation

A specification could appear portable while requiring undocumented setup or losing the meaning of its stored outcomes.
