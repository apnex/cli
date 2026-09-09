# Use the Rust JSON authoring CLI

The first application constructs JSON through contextual commands and checkpoints every acknowledged edit.\
The saved session contains the unfinished candidate, accepted baseline, context, original task, revision, and last successful mutation receipt.\
A session checkpoint and an exported JSON document serve different purposes.\
The optional [constraints layer](../constraints/USE.md) adds schema guidance and validates whole documents on commit while retaining the same editing and checkpoint operations.\
The [import and separate-use walkthrough](IMPORT-USE.md) transfers an authored CLI specification into a new editable session and activates it there.

## Build and start

Run from the repository root with Rust and Cargo available, a C linker, and a local Linux filesystem supporting file locks, atomic rename, hard links, and file and directory synchronization.\
The tested toolchain is declared in [rust-toolchain.toml](../../rust-toolchain.toml), with exact observations in the [publication verification](../evidence/publication/verification.json).\
Earlier toolchains and other operating systems have not been qualified by this slice.

```sh
cargo build --locked --bin cli
demo_dir=$(mktemp -d)
target/debug/cli --definition docs/authoring/operations.json \
  --session "$demo_dir/catalog.session.json" \
  --create --intent-file docs/authoring/acceptance/SERVICE-CATALOG.md
```

The task file supplies original intent; it is not imported as candidate JSON.\
New sessions start with an empty object.\
The declaration supplies the operation surface and is identified by its exact byte digest.\
Creation refuses an existing checkpoint.

At an interactive terminal, Tab completes commands, paths, value kinds, and declared choices.\
The prompt shows context, revision, and `*` when the draft differs from the accepted baseline.\
Menus with additional matches say so; narrow the prefix or use `complete` with an offset to inspect further children.\
Large human-readable responses fall back to compact JSON to preserve the output bound and complete data.

---

## Construct in context

These lines are entered inside the CLI:
```text
set /project string atlas
set /services array
append /services object
edit /services/0
set ./name string api
set ./enabled boolean true
set ./quota number 1.2300
set ./ports array
append ./ports number 8080
show
up
top
diff
commit
```

Use `help` or `help set` to inspect the loaded contracts.\
The complete service-catalog task also exercises unusual keys, draft export, resumption, and discard; its [paired journey](acceptance/authoring-cases.json) supplies each terminal command.

| Input | Meaning |
|---|---|
| `.` | Current value. |
| `./name` | A member relative to the current context. |
| `/services/0` | A path from the root; the parent determines whether `0` is an index or a literal object key. |
| `""` | The document root. |
| `/` | The empty-key member of the root object. |
| `./` | The empty-key member of the current object. |
| `~0`, `~1` inside a pointer | Literal tilde and slash, decoded once. |

Values always name a kind: `object`, `array`, `string`, `number`, `boolean`, or `null`.\
Containers start empty; build their children with operations.\
Quote complete tokens using JSON string escapes when they contain spaces or controls:
```text
set "/release note" string "first line\nsecond line"
set /looks-boolean string true
set /precise number 9007199254740993
set /budget number 1e400
```

Numbers retain their exact tokens, including negative zero and trailing fractional zeros.\
No shell expansion runs inside command input.\
`set` can create a final object member; intermediate containers must already exist.\
Array replacement, append, and insertion are separate operations.

---

## Accept, export, and resume

`commit` accepts the candidate as the local baseline.\
`discard` restores that baseline and returns context to root.\
`save /absolute/path/catalog.json` exports the candidate without accepting it or changing the session revision.\
Save creates a destination, accepts identical existing bytes, and refuses different existing content.\
`import /absolute/path/catalog.json` explicitly replaces the whole candidate and returns focus to root; it leaves acceptance, attached schema, and any active CLI interface unchanged.\
Inspect `diff`, repair the imported draft if needed, then use `commit` to accept it.\
An imported source can contain up to 2 MiB including formatting; the resulting document retains the existing limits.

Ctrl-D closes the session.\
Reopen the same checkpoint with the same declaration bytes:
```sh
target/debug/cli --definition docs/authoring/operations.json \
  --session "$demo_dir/catalog.session.json"
```

The startup view includes original intent, current state, and the last saved receipt.\
Only one cooperating process can own a session at a time.\
Its stable sibling lock file remains on disk after exit; an inactive lock file does not mean a process owns the lock.\
Checkpoint corruption is reported explicitly and never resets the draft automatically.

---

## Atomic batches

```text
batch
set /labels object
set /labels/environment string dev
end
```

The prompt and input events mark these edits as unsent until `end`.\
Only root-based primitive edits belong in a batch.\
The preview supports completion for newly entered paths, while the checkpoint remains unchanged.\
`cancel`, Ctrl-C, or closing the session drops unsent input.\
A submitted failure identifies its zero-based edit index and preserves the entire prior checkpoint.\
A successful batch consumes one revision.

---

## Machine and command streams

`--machine` accepts complete JSON requests and emits JSON events, one per line.\
It emits `session_open` before reading a request.\
The [request contract](SESSION-CONTRACT.md#request-and-result-contracts) defines the envelope, typed paths, response fields, and validation order.\
Read requests omit `expected_revision`; state changes and exports include the last observed revision string.

`--commands` accepts the same terminal language and emits structured events, making terminal syntax scriptable without a TTY.\
For example, against a newly created session:
```sh
target/debug/cli --definition docs/authoring/operations.json \
  --session "$demo_dir/script.session.json" --create \
  --intent-file docs/authoring/acceptance/SERVICE-CATALOG.md --commands <<'COMMANDS'
set /project string atlas
set /quota number 1.2300
show
COMMANDS
```

Machine requests and terminal commands use one dispatcher.\
Malformed lines are rejected while later lines remain usable.\
A stream can exit successfully after ordinary rejected operations; inspect each response's status.\
Startup failure, transport failure, or unresolved checkpoint publication exits nonzero.\
Standard error carries transport diagnostics; machine standard output contains only protocol events.

If a reply is lost, reopen and inspect the receipt before retrying.\
The exact last successful state request replays its saved response without another edit or revision.\
Reusing its identifier for different input fails, and an older request is not rebased automatically.\
After `PERSISTENCE_UNCERTAIN`, close and reopen before another write.\
After `EXPORT_UNCERTAIN`, inspect or repeat the same export at the same revision.

---

## Bounds and current scope

| Resource | Bound |
|---|---|
| Compact candidate or accepted JSON | 1 MiB each. |
| Document depth | 64; root is depth zero. |
| Decoded string, object key, number token, original task | 64 KiB each. |
| Request line | 2 MiB. |
| Batch | 1 through 256 edits. |
| Checkpoint including receipt, response, startup event | 8 MiB each. |

Known failures reject before publication.\
Result collections stop at their size bound while being assembled; they do not silently omit changes.\
Fault tests cover local process interruption and filesystem-call failures, not power-loss behavior of every storage device.\
The lock and prior-byte comparison do not provide compare-and-swap against a simultaneous writer that ignores the lock.

This release uses a hand-authored bootstrap declaration with registered Rust authoring behavior.\
Compatible declaration data can change operation names, terminal words, argument ordering, defaults, and supported constraints on arguments.\
The [composition profile](../composition/USE.md) supports authored definitions and labeled mocks; the [constraints profile](../constraints/USE.md) supports schema enforcement.\
[Local component assembly](../components/USE.md) and [granted JSON file reads](../connected/USE.md) are implemented within their selected contracts.\
Broader package resolution and executable or remote providers require a concrete consumer on the [board](../BOARD.md).

Correction: the earlier sentence "General component assembly and real hooks remain on the board" predated those bounded capabilities and remained after their completion.\
It is superseded by the two explicit scopes above; no general executable-hook capability is implied.

---

## Verify and remove

```sh
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo fmt --all -- --check
cargo run --locked --manifest-path tools/scaffold/Cargo.toml -- --check
cargo test --locked --manifest-path tools/scaffold/Cargo.toml
```

The first command verifies the normal build, including inert fault controls.\
The second enables explicit fault instrumentation and three separate registration-drift probe binaries for tests.\
Use the normal `cli` binary for authoring; test controls are absent from its default build.\
The scaffold commands check documentation consistency separately from application assertions.\
The [implementation record](CLI-002.md) identifies retained evidence and its limits.

The application installs no service and opens no network connection.\
To remove a local build, use `cargo clean`.\
Retain session checkpoints and exports you intend to keep; remove their files and inactive sibling lock files only when their sessions are closed.
