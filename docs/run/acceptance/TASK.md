# Direct run acceptance task

Use a CLI specification produced by the existing command-only authoring workflow as an ordinary CLI.
Running the exported connected catalog's `services inspect` command with a file grant must return the source JSON and exit successfully.
Running its `services quota 1e400` command must return the exact mock result, label the simulation, and exit successfully.
Running `services restart` must report its unbound implementation and exit unsuccessfully.
None of these user commands may require an `invoke` prefix or manual authoring-session setup.

The specification must work as a one-shot process and in an interactive context-aware shell.
Help must describe actual commands, required parameter types, and binding requirements.
Tab completion must follow context and suggest allowed boolean values.
Tree output must show usable direct command paths.
Domain commands named after authoring controls must remain reachable.

Use two assembled components to prove that the displayed local paths agree with actual routing.
Use optional persistence to modify mock state, reopen it, reject a changed specification, resume without the source, and export the active history.
A failed invocation must preserve the prior checkpoint, including navigation state.
Quoted strings and exact numeric spelling must survive both OS argv and interactive input.
Invalid commands, extra or mistyped arguments, missing grants, and unbound operations must produce meaningful nonzero exit codes.
A later successful piped command must not erase an earlier error from the exit status.

Compare behavior with the existing authoring invocation engine, and test publication failure and uncertainty at observed boundaries.
Exercise an actual terminal, including completion and context navigation, and retain its transcript.

## Mechanics, rationale, and consequence

### Mechanics

Invoke actual CLI processes with independently specified commands and compare output, exit status, and persisted state.

### Rationale

Direct verbs are the user's selected consumption experience for the specifications already authored by the kernel.

### Consequence of violation

A parser demonstration could otherwise stand in for a usable CLI or conceal changed state semantics.
