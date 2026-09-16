# Author and use native output views

A view selects typed values from a command result and formats a table inside the Rust binary.
The first consumer reproduces AGP's connection and route views using recorded JSON fixtures.
It does not contact an AGP service.

Run these blocks from the repository root in the same POSIX shell.
Building needs Rust/Cargo and a C linker; the construction example uses ordinary filesystem utilities to prepare separate directories.
Projection and rendering require no Bash, jq, or column executable.
The [format reference](FORMAT.md) describes the authored data, and the [results](RESULTS.md) identify the checks and limits.

## Preview a recorded document

```sh
cargo build --locked --release --bin cli
./target/release/cli render docs/output-views/acceptance/agp.json connections \
  docs/output-views/acceptance/fixtures/connections-cases.json
./target/release/cli render docs/output-views/acceptance/agp.json routes \
  docs/output-views/acceptance/fixtures/routes-cases.json
```

Tables go to stdout; stderr identifies these as document previews with no binding invoked.
The connection columns include identity, peer, state, supplied uptime, and remaining hold time.
The route columns include selection, endpoint, next hop, path, eligibility, and reason.
An empty result still prints its headings.

## Call an ordinary verb with a declared view

```sh
./target/release/cli run --table \
  --grant-json-read agp.connections docs/output-views/acceptance/fixtures/connections-cases.json \
  docs/output-views/acceptance/agp.json connections.list
./target/release/cli run --table \
  --grant-json-read agp.routes docs/output-views/acceptance/fixtures/routes-cases.json \
  docs/output-views/acceptance/agp.json routes.list
./target/release/cli run docs/output-views/acceptance/agp.json :views
```

The two verbs use the existing native JSON-file reader with explicit process-local grants.
The result document and its observation metadata remain in the invocation receipt.
`--table` selects the command's declared view; `--view routes` selects a named view explicitly.
Omitting both retains the existing original-JSON output.

Add `--json` to receive the complete structured response with `result.presentation` alongside `result.invocation`.
Presentation contains ordered column IDs, exact `rows_json_text`, cleaned `display_rows`, and `table_text`.
Formats change display cells, so a typed duration stays numeric and a joined path stays an array in projected rows.

## Construct, export, and import without editing JSON

The [authoring recipe](acceptance/agp.commands) constructs both commands and every view expression through `set`, `edit`, `append`, `up`, and `top`.
It commits, saves a standalone definition, activates it, and exports an interface bundle.
No braces or brackets occur in the recipe.

```sh
view_repo=$PWD
view_cli="$view_repo/target/release/cli"
view_demo=$(mktemp -d "$view_repo/target/output-views-XXXXXX")
mkdir "$view_demo/author" "$view_demo/consumer" "$view_demo/receiver" "$view_demo/empty-path"
(
  cd "$view_demo/author"
  "$view_cli" --definition "$view_repo/docs/authoring/operations.json" \
    --session author.json --create --intent-file "$view_repo/docs/output-views/acceptance/TASK.md" \
    --compose --commands < "$view_repo/docs/output-views/acceptance/agp.commands" > author.jsonl
)
cp "$view_demo/author/agp.json" "$view_demo/consumer/spec.json"
cp "$view_demo/author/agp-interface.json" "$view_demo/consumer/interface.json"
(
  cd "$view_demo/receiver"
  "$view_cli" --definition "$view_repo/docs/authoring/operations.json" \
    --session receiver.json --create --intent-file "$view_repo/docs/output-views/acceptance/TASK.md" \
    --compose --commands > import.jsonl <<'COMMANDS'
import ../consumer/spec.json
commit
save received.json
activate
export-interface received-interface.json
COMMANDS
)
cmp "$view_demo/author/agp.json" "$view_demo/receiver/received.json"
```

The standalone definition and interface bundle both include the view definitions.
The bundle additionally carries interface state and origin; grants remain process-local.
The Rust acceptance test checks every authoring response, exported bytes, both transferred forms, and the resulting tables.

## Run transferred views without external executables

```sh
PATH="$view_demo/empty-path" "$view_cli" run --table \
  --grant-json-read agp.connections "$view_repo/docs/output-views/acceptance/fixtures/connections-cases.json" \
  "$view_demo/receiver/received.json" connections.list > "$view_demo/connections.txt"
PATH="$view_demo/empty-path" "$view_cli" run --table \
  --grant-json-read agp.routes "$view_repo/docs/output-views/acceptance/fixtures/routes-cases.json" \
  "$view_demo/consumer/interface.json" routes.list > "$view_demo/routes.txt"
cmp "$view_demo/connections.txt" "$view_repo/docs/output-views/acceptance/expected/connections-cases.txt"
cmp "$view_demo/routes.txt" "$view_repo/docs/output-views/acceptance/expected/routes-cases.txt"
```

The empty `PATH` applies only to each CLI process.
The later `cmp` commands check its output against the unchanged AGP reference tables.

## Re-render a retained result

```sh
"$view_cli" run --table --session "$view_demo/run.json" \
  --grant-json-read agp.connections "$view_repo/docs/output-views/acceptance/fixtures/connections-cases.json" \
  "$view_demo/consumer/spec.json" connections.list > "$view_demo/first.txt"
"$view_cli" run --session "$view_demo/run.json" - :render connections > "$view_demo/retained.txt"
cmp "$view_demo/first.txt" "$view_demo/retained.txt"
"$view_cli" run --json --session "$view_demo/run.json" - :status > "$view_demo/receipt.json"
printf 'View workflow retained in %s\n' "$view_demo"
```

`:render` uses the last retained invocation and labels it historical; it needs neither a grant nor the original input/specification files.
It does not invoke a provider, change the revision, or replace the receipt.
Mock results keep their simulated label.

If a command succeeds but its view fails, the process exits 1 and reports a presentation error while preserving the successful invocation.
Use `:status` to inspect the receipt or `:render` with a compatible view.
Repeating the verb is not the repair operation.
Use `--session` when you need that receipt to survive process exit; an ephemeral run retains it only while its process remains open.

## Verify and extend

```sh
cargo test --locked --test cli_output_views
```

Reusable components can include v3 views; assembly scopes view IDs and command references without rewriting paths into result documents.
Extend an authored definition with another view, export it, then preview it against a captured result before choosing it for a command.
The first vocabulary has explicit type, work, and output limits.
Sorting, grouping, arbitrary scripts, terminal-width truncation, themes, and live network providers are outside this increment.
