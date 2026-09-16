# Configurable output-view contract

**Status: implemented for CLI-013; [measured results](RESULTS.md) record acceptance and limits.**
Authority: [owner selection](../context/cli-013-selection.json) and [acceptance task](acceptance/TASK.md).
The contract was selected before implementation; its build-order section retains that plan.

## From-state and target

The [source inspection](../research/command-output-views/CURRENT-CLI.md) records the starting state.
The target is one native Rust projection/formatting path serving both AGP tables, saved invocation results, and explicit document previews.
View definitions are data constructed through existing authoring operations and included in specification transfer.

---

## Definition and interpretation

`cli-definition-v3` adds a nonempty `views` map and optional command `view` references.
Older formats omit both fields and retain their prior canonical serialization.
A view contains explanatory text, an input requirement expression, a row expression, and ordered columns with stable IDs, headings, value expressions, and formats.
Activation checks names, references, expression shapes and scope, duplicate columns, and structural limits.
Unknown fields and unsupported operators reject activation.
Components may carry plain v3 definitions; assembly prefixes view IDs and command references while retaining result-relative paths.

Expressions are bounded data with an `op` discriminator: `root`, `row`, `item`, `literal`, `coalesce`, `eq`, `and`, `is`, `if`, `first`, `any`, and `concat`.
Paths use the existing typed key/index segments.
`first` and `any` evaluate predicates with an explicit item scope while retaining the outer row and root.
Missing paths remain distinct from explicit null values; `coalesce` declares whether false also counts as absent.
Equality preserves existing exact document equality, including numeric token spelling; numeric comparison/sorting is outside this first vocabulary.
The input requirement must yield true; rows must yield an array.

Formats are plain values, duration hours/minutes/seconds, ceiling seconds clamped at zero, and joined arrays.
Duration formats operate on exact decimal tokens, with bounded output digits; they do not convert through floating point.
Negative elapsed durations reject, while remaining seconds clamp at zero.
Non-number duration inputs and non-array joins use the configured fallback.
Table cleaning replaces control and directional formatting characters, escapes backslashes, and measures Unicode display width.
The initial layout uses full column widths and two spaces between columns, matching the frozen AGP plain tables.
It does not silently truncate values to terminal width.

Limits are part of the implementation's public error behavior: 64 views, 32 columns per view, expression depth 16, 2048 expression nodes per view, 4096 rows, 100000 evaluation steps, 16 MiB cumulative evaluated-value copying, 4 MiB rendered/projected output, and 4096 digits for duration arithmetic.
Existing document, scalar, request, checkpoint, and response limits also apply.

---

## Consumption and recovery

`cli run --table` uses the invoked command's declared view; `--view <name>` selects a named view explicitly.
Plain run mode continues to emit the original JSON payload unless table selection is requested.
`--json` continues to emit the complete response; with a selected view it additionally exposes presentation data or its error without changing the invocation outcome.
Definition-owned help and discovery expose view availability.
`:views` lists view definitions; `:render <name>` renders the last successful invocation without executing it or changing saved state.
Colon controls retain their namespace separation from configured verbs.

`cli render [--json] <spec.json> <view> <document.json>` previews a view over an explicit bounded local document without opening a session or invoking a binding.
Preview output is identified as document presentation, not an external service observation.
Display rows and exact typed projected rows are exposed separately; the original invocation remains in its existing receipt.

Unknown view selection rejects before invocation.
Input-dependent projection failure after an invocation reports that the invocation succeeded and rendering failed; it preserves the receipt, original result, and effect classification.
Recovery inspects or re-renders the retained result instead of executing the verb again.
Output transport failures retain the existing stop/reopen contract.
Simulation labels remain runtime-owned in both tables and structured output.
Rendering is deterministic for a fixed definition, input, and renderer; it reads no clock, locale, network, or process output.

---

## Build order and verification

1. Capture the existing source/binary and AGP reference inputs/outputs before implementation.
2. Validate definitions and implement pure expression evaluation, cell formats, and bounded table layout.
3. Integrate run selection, discovery, saved-result rendering, and document preview over that shared implementation.
4. Execute the contextual construction/transfer journey and compare against the frozen oracle.
5. Exercise invalid types, failures after successful effects, exact numbers, assembly, output limits, empty PATH, and the required contributor checks.

Each numbered acceptance observation in the [task](acceptance/TASK.md) is a binary exit criterion; task counts alone do not close the change.
The [design audit](AXIOM-AUDIT.md) carries the implementation guardrails.

---

## Fence, costs, and non-claims

This slice covers the two concrete AGP views and shared mechanisms they require.
The Rust binary replaces runtime Bash, jq, and column usage for those views.
Two small Rust libraries already present in the dependency graph supply arbitrary-size integer arithmetic and Unicode width; no executable helper is used.
There is one application implementation and no public crate/API compatibility guarantee.

Full jq semantics, arbitrary scripts, new network providers, grouping, sorting, streaming/watch mode, auto-width truncation, color themes, and changing default output are outside this slice.
Matching frozen plain tables does not establish live network behavior or all terminal layouts.
This task supplies local executor evidence and upstream reference comparisons, not independent certification or deployment proof.
