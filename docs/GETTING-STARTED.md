# Get started

Build a document, construct a CLI specification through commands, and use the resulting interface in a separate run session.\
The examples use a disposable working directory and the repository's existing authoring recipes.\
They exercise simulated behavior; the optional connected read capability has its own [grant walkthrough](connected/USE.md).

## Prerequisites

Use a local Linux filesystem supporting advisory file locks, atomic same-directory rename, hard links, and file and directory synchronization.\
Use directories you control for source documents, checkpoints, and exports; this is a local application, not a sandbox for hostile filesystem writers.\
Other operating systems, network filesystems, and hardware power-loss behavior are not qualified.

Install Rust and Cargo with a C linker using the [official Rust installation guide](https://www.rust-lang.org/tools/install).\
The exact tested toolchain is retained in the [publication verification](evidence/publication/verification.json); an earlier minimum supported Rust version has not been established.\
First-time dependency fetching requires network access; authoring and run mode do not require a network service or LLM account.\
These shell examples also require `jq` to check structured outcomes.

Run the shell blocks below from the repository root in the same shell.\
The `[simulated]` label is part of interpreting a mock result, even when stdout is redirected.

---

## Build

```sh
set -eu
getting_started_repo=$PWD
cargo build --locked --release --bin cli
getting_started_cli="$getting_started_repo/target/release/cli"
getting_started_demo=$(mktemp -d)
"$getting_started_cli" --help
```

The source build includes its configured-run declarations in the binary.\
Authoring sessions explicitly select the authoring declaration from the checkout.

### Optional installation

For a binary on your Cargo executable path, run this independently from the repository root:
```sh
cargo install --locked --path . --bin cli
```

Cargo installs the executable in its configured installation root.\
Check that root's `bin` directory is on your shell's `PATH`.\
Keep the source checkout when using its authoring declarations, example recipes, or contributor checks.

---

## Author JSON in context

Create and accept a draft through commands:
```sh
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
```

Each value has an explicit kind, so strings that resemble numbers or booleans keep their intended type.\
No JSON containers are handwritten in this input.\
`commit` accepts the local draft; every acknowledged edit was already checkpointed.\
The [authoring guide](authoring/USE.md) explains paths, `save`, `import`, atomic batches, recovery, and bounds.

Reopen and inspect the same draft:
```sh
"$getting_started_cli" --definition docs/authoring/operations.json \
  --session "$getting_started_demo/document.session.json" --commands \
  > "$getting_started_demo/reopened.events.jsonl" <<'COMMANDS'
show
COMMANDS
jq -es 'all(.[]; .event == "session_open" or .status == "ok")' "$getting_started_demo/reopened.events.jsonl"
```

For interactive editing, omit `--commands` and input redirection when launching from a terminal.\
The shell supplies contextual Tab completion and discovery through `help`.

---

## Construct and export a CLI

The [connected catalog recipe](connected/acceptance/catalog.commands) progressively builds contexts, typed command parameters, an explicit mock, a declared file-read requirement, and an unbound operation.\
It ends with `commit` and `save connected-cli.json`.\
The task file preserves why that interface is being authored.

Run the recipe in a separate authoring directory:
```sh
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
```

You can enter those same recipe lines interactively and inspect or revise the document between operations.\
The serialized specification is an output of that workflow.

---

## Run the exported interface

Call its verbs directly in the consumer directory:
```sh
(
  cd "$getting_started_demo/consumer"
  "$getting_started_cli" run spec.json :tree > tree.txt
  "$getting_started_cli" run spec.json services --help > help.txt
  "$getting_started_cli" run spec.json services quota 12 > quota.json 2> quota.stderr
)
cat "$getting_started_demo/consumer/tree.txt"
cat "$getting_started_demo/consumer/quota.stderr"
test "$(cat "$getting_started_demo/consumer/quota.json")" = '{"quota":12}'
```

The quota result is simulated.\
The declared `inspect` operation requires a file grant for a fresh connected read, while `restart` remains discoverable but unbound.\
Omitting the command opens a contextual shell on a terminal; piped input accepts the same command language.

Navigate and retain state in a named session:
```sh
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
```

The [run guide](run/USE.md) covers one-shot versus contextual paths, literal arguments, output, exit codes, state transfer, and connected observations.\
The [documentation index](README.md#choose-a-workflow) points to schema construction and component assembly.

---

## Remove

For a binary installed with Cargo, remove that installation:
```sh
cargo uninstall programmable-cli
```

For a local source build, remove only its build products from the repository root:
```sh
cargo clean
```

These commands do not delete the example directory printed above, user checkpoints, or exported definitions.\
Preserve those files until you no longer need their data or provenance.\
Close every process using a checkpoint before removing its directory; a remaining sibling lock file alone does not mean a lock is held.\
Temporary run sessions clean up on normal exit; a killed process can leave a temporary directory.

---

## Mechanics, rationale, and consequence

### Mechanics

Construct data through typed commands, export a definition, and consume it from a separate directory using direct verbs.

### Rationale

The construction workflow and the resulting CLI are independently useful and share a portable specification.

### Consequence of violation

A demonstration that depends on the author's hidden setup does not establish that another user can construct or consume the interface.
