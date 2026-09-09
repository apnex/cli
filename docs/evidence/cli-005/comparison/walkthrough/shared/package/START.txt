# Continue the worker-platform prototype

Work from this package directory on the platform recorded in `manifest.json`.
The supplied executables and declaration identify the compatible runtime.
The package contains the complete original catalog task and second task; repository links inside the original task are provenance, not required setup inputs.

## Read intent and test compatibility

Read both tasks:
```sh
cat CATALOG-TASK.md
cat TASK.md
```

The recorder creates `work.session.json` from the immutable handover only if no working copy exists.
It preserves previous work on subsequent invocations.
Use one working copy at a time; copies are independent trials and do not merge changes.
Inputs, outputs, failures, and process timing are retained under `logs/`.

Exercise the incompatible declaration and abandoned stale request:
```sh
./receiver probe .
```

Expect `DEFINITION_MISMATCH` and `REVISION_CONFLICT`, with no checkpoint change.
The recorder checks those outcomes and exits unsuccessfully if either expectation fails.
The abandoned deletion is not part of the new task and must not be refreshed and executed.

## Inspect the unfinished work

Inspect intent, constraints, both document versions, and the active interface, then attempt the invalid commit:
```sh
./receiver run . <<'CLI'
status
help
schema
guide /mock_state/queue/paused
show "" candidate
show "" accepted
diff
discover
tree
commit
CLI
```

The final commit is expected to reject the string value in `queue.paused` without changing the checkpoint or accepted document.
The draft's authoring location and active interface location differ; each response header states them.
The recorder exits on transport failure; individual command rejections are JSON events that must be read even when the process exits successfully.

## Finish through commands

Use further `receiver run` invocations with terminal command input to repair and complete `TASK.md`.
These operations build data through typed values: `set`, `append`, `insert`, `delete`, `edit`, `up`, and `top`.
Each successful state operation is checkpointed; EOF closes a recorder invocation.
All candidate and definition changes must go through this interface.
Do not modify JSON files or replace the checkpoint with hand-authored content.

| Input | Meaning |
|---|---|
| `help set` | Discover the operation's actual arguments. |
| `set ./key string text` | Set a member relative to the authoring context. |
| `set /key boolean true` | Set an absolute member with an explicit JSON kind. |
| `set /container object` | Create an empty object, then populate its members. |
| `append /items object` | Append an empty object to an existing array. |
| `""` | Document root; `/` instead identifies the empty-key member. |
| `commit` | Validate and accept the authored document. |
| `activate` | Explicitly load the candidate as a CLI definition. |
| `enter queues` | Select interface context without moving document context. |
| `invoke pause true` | Invoke the configured boolean mock after activation. |
| `tree` | Read the active verb structure. |

After completing the requested exports, retain a plain-text account of unmet requirements, repairs, and any additional context or human input you needed.
Do not claim real service or queue effects from these mocks.

## Mechanics, rationale, and consequence

### Mechanics

The CLI owns authoring and runtime state; the recorder retains the trial interaction without supplying a solution.

### Rationale

Another actor should be able to continue from explicit intent and structured work without the originating conversation.

### Consequence of violation

Editing artifacts directly or consulting the hidden completion recipe would invalidate this continuation trial.
