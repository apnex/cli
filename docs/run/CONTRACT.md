# Direct CLI run contract

**Status: implemented within the selected CLI-008 scope; [measured results](RESULTS.md) retain acceptance and limits.**
The [approval](../context/cli-008-approval.json) fixes direct configured verbs as the user-facing experience.
The [task](acceptance/TASK.md) defines acceptance independently of the implementation.

## Launch and interface

`cli run [options] <spec.json> [context ... command arguments ...]` loads a plain definition, assembled definition, or interface export.
It uses the embedded composition declaration; users supply neither the bootstrap operations file nor authoring setup commands.
With a command, it performs one invocation and exits.
With no command, a terminal receives a contextual REPL and piped input is interpreted as command lines.
Empty piped input prints root help.
Existing authoring launches remain available.

Options are `--json`, `--session <checkpoint>`, and repeated `--grant-json-read <capability> <file>`.
They occur before the first configured command/context word; `--` ends launcher option parsing.
`--help` before the specification prints launcher help.
After the specification, `--help` at a context or immediately after a command requests generated help.
An argument separator `--` preserves a literal help-like argument; once command arguments begin, they are data.
OS argument strings are never joined and reparsed; interactive input retains the existing quoted-token grammar without shell expansion.

One-shot paths start at root.
Interactive paths start at the selected context; entering a context with no command changes the prompt and navigation state.
A qualified invocation does not implicitly move that context.
The current scalar signature supports ordered required string, number, and boolean arguments; named options, defaults, optional parameters, and API mapping remain outside this increment.

## Routes and controls

Context ownership generates command paths.
Plain definitions use their context IDs as context words.
For assembled definitions, a child ID prefixed by its parent's ID and a period is displayed using the remaining local suffix.
The same local words drive routing, help, tree output, and completion.
Command/context collisions or duplicate child words reject run activation with `AMBIGUOUS_RUN_ROUTE`; nothing is silently hidden by precedence.
Run-mode rejection does not make the definition invalid for the authoring workspace.

Control words use a colon, which configured identifiers cannot contain:

| Control | Behavior |
|---|---|
| `:help [context ... command]` | Generated context or command help, including parameters and binding requirements |
| `:tree` | Entire configured verb tree without the `invoke` prefix |
| `:up`, `:top` | Navigate the parent or root context |
| `:status` | Current interface, process grant availability, and historical outcome |
| `:export <file>` | Export the active interface through existing create-only publication |
| `:exit` | Close the command stream or interactive session |

Configured commands named `set`, `help`, `tree`, `exit`, or `invoke` remain ordinary domain commands.
The run surface exposes no document-authoring operations implicitly.
Tab completion follows the active context and declared command signatures, including boolean values, and replaces only the token being edited.
The prompt identifies the active definition and selected context, without implying that all commands are simulated.

## Shared execution and persistence

All configured calls use the existing composition invocation handler, provider implementations, private mock execution, exact scalar constructors, and checkpoint receipt publisher.
The handler gains a qualified `context-id/command-word` target; identifiers cannot contain the separator.
Local authoring invocations remain supported.
One configured invocation is one checkpoint transition; no hidden navigation commits precede it.
Saved receipts validate the actual target context rather than assuming it equals the current navigation context.

Without `--session`, a run has fresh state initialized from the supplied definition or interface export.
It uses a private temporary checkpoint directory that is removed on normal exit; forced process termination may leave that temporary directory.
`--session` creates a persistent composition checkpoint if missing or reopens it if present.
The receiving session's task text is explicitly generated run provenance, not a fabricated original authoring task.
The imported interface retains its original provenance.
Reusing a persistent checkpoint with a supplied specification requires the same definition digest; matching input does not reset recorded state.
A changed definition reports `RUN_SPEC_MISMATCH` without rewriting the saved checkpoint.
Use `-` in place of the specification to resume an existing persistent checkpoint without its source file.
The first run profile uses composition checkpoint format 2; interface exports are the transfer boundary for other profiles.

Grants remain process-local and must be supplied for fresh connected reads.
`:status` reports history without refreshing it; `:export` transfers history without authority.
Underlying exact receipt replay remains available through the canonical request interface; repeating a one-shot shell command is a fresh request.
Publication uncertainty stops execution and requires reopen before another invocation.
Previous binary/declaration pairs are preserved before changing the embedded invocation metadata.

## Output and exit status

Successful invocations write only the exact JSON result text and a newline to stdout by default.
Simulated results are labeled on stderr so a mock cannot silently resemble a connected observation.
Connected output retains its full provenance in the receipt and `--json` output.
Errors go to stderr with no success payload on stdout.
Explicit help, tree, and status requests produce their requested views; startup emits no authoring checkpoint dump.
`--json` emits structured response events with runtime-owned outcome labels, request identity, revision, and errors.
Generated help and tree views also have structured forms; there is no implicit startup event.
Human-readable metadata escapes terminal control characters.

| Exit | Meaning |
|---|---|
| 0 | Requested work succeeded, including help and explicitly labeled mocks |
| 2 | Launcher usage, route, or argument error |
| 1 | Definition, source, grant, unbound operation, provider, storage, or delivery failure |

Piped and interactive sessions retain a nonzero outcome if any submitted command fails, even if a later command succeeds.
They continue after ordinary errors to permit correction; uncertain publication or transport failure ends execution.
Ctrl-C drops unsent interactive input and Ctrl-D closes the session.
Output bounds are checked before writes; a delivery failure after publication cannot be reported as an ordinary mutation-free rejection.

## Acceptance predicate

The original exported mock and connected definitions work without authoring controls or `invoke` in one-shot and contextual input.
Expected outputs, precise numbers, stderr labels, generated help/tree/completion, grants, invalid inputs, literal argv strings, and nonzero exits are asserted independently.
Assembled paths use local context words consistently.
Configured words that resemble authoring controls remain reachable; ambiguous routes reject before creating a run session.
Persistent qualified invocations preserve navigation and unrelated state, failures preserve checkpoint bytes, source-free reopen retains outcomes, and changed definitions do not reset saved state.
Publication faults and actual terminal interaction exercise the same runtime path.
Execute the documented workflow literally and retain failures and source/binary identities.

## Mechanics, rationale, and consequence

### Mechanics

Project the loaded definition into a direct command surface and lower calls to the existing transactional dispatcher.

### Rationale

The exported specification must become a usable CLI without requiring its operator to understand the authoring workspace.

### Consequence of violation

A reusable specification would still require a wrapper vocabulary, or a new frontend could silently change invocation and recovery semantics.
