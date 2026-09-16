# AGP template and table behavior

**Tier 2: source inspection and offline measurements, 2026-09-16.**

Inspected `/home/apnex/taceng/agp` at `da1a03a370f522bfb2a3afc66fb781b295cb6af7`.
The worktree was clean and `git ls-remote origin refs/heads/main` returned that same revision.
Scope: the two existing CLI projections and shared renderer. No management service was queried.

## Source findings

| Surface | Measured mechanism | Source at the inspected revision |
|---|---|---|
| Dispatch | `connections.list` and `routes.list` select fixed command scripts. Each script supplies a fixed driver, template, and projection name. | [Dispatcher](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/agpctl), [connection command](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/cmd.connections.list.sh) |
| Output modes | A driver obtains one JSON payload. Default output passes it through `jq -f` and then the renderer. `--json` bypasses projection and passes the payload through `jq .`; this reserializes JSON. | [Command execution](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/lib/command.sh#L91) |
| Connection rows | Require the expected version, kind, and array. Select nested fields and fallback identities; format supplied elapsed milliseconds as total hours/minutes/seconds; select an armed hold timer and round remaining milliseconds up to seconds. | [Connection template](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/tpl/tpl.connections.list.jq) |
| Route rows | Require the expected version, kind, and arrays. Mark a candidate by matching its ID against the root `selected` array; format a local or session next hop; join path elements with `>`. Route selection itself is supplied by the document. | [Route template](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/tpl/tpl.routes.list.jq) |
| Row values | Templates produce display strings. `clean` converts null to an empty string and other nonstrings with `tostring`; it replaces controls and specified directional formatting characters with spaces. | Both templates, `clean` |
| Layout | Column names and order are hard-coded in a shell `case`. Headers are uppercase. Rows become TSV, then pass through `column -t` when available; absence of `column` leaves TSV. | [Renderer](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/lib/render.sh#L9) |
| Terminal output | Renderer sanitizes cells again. Cyan headers require a terminal and absence of `NO_COLOR`. Empty arrays still print headers. | Renderer, lines 41-87 |
| Configuration boundary | Option parsing accepts JSON output, URL, and help. It has no caller-supplied template, columns, sorting, filtering, grouping, or pagination options. The renderer accepts only the two projection names. | [Options](https://github.com/apnex/agp/blob/da1a03a370f522bfb2a3afc66fb781b295cb6af7/cli/lib/command.sh#L16), renderer |

## Executed observations

Existing tests were run unchanged:

```sh
node --test cli/test/unit/connections-template.test.js \
  cli/test/unit/route-template.test.js \
  cli/test/unit/cli-renderer.test.js
```

**13 passed, 0 failed, process exit 0.**
The [raw test log](../../evidence/command-output-views/agp-template-renderer-tests.log) retains the test names and outcomes.
These cover empty inputs, optional fields, supplied timer durations, uptime over one day, route markers, rejected versions/kinds, absent `column`, and hostile terminal text.

Separate [offline probes](../../evidence/command-output-views/agp-output-probes.json) retain exact inputs, stdout, stderr, exits, source hashes, and tool versions.
Environment: Node v24.12.0, jq 1.6, and `column` from util-linux 2.34; output was captured through pipes with `NO_COLOR=1`.

| Probe | Observation |
|---|---|
| Existing connection fixture through template and renderer | Both stages exit 0; elapsed time prints `01:00:00`, hold TTL `21s`, and hostile strings remain one physical row. |
| Existing route fixture through template and renderer | Both stages exit 0; the selected local route has `>`, the session next hop is `leaf.alpha@75c4ae`, and the path is `leaf.alpha>hub.local`. |
| Synthetic numeric cell | Renderer exits 0 but input `9007199254740993` prints `9007199254740992` in this jq 1.6 pipeline. This is a generic rendering stress case, not a valid AGP numeric session identity or a finding about every jq version. |
| Long ASCII cell, non-terminal output, `COLUMNS=32` | Renderer exits 0; the longest output line contains 223 characters. This probe makes no claim about terminal behavior or other `column` versions. |

First data row from the connection fixture, with its header:

```text
SESSION_ID   REMOTE_NODE  DIRECTION  STATE        UPTIME    TTL  LAST_EVENT
75c4ae       leaf.alpha   inbound    Established  01:00:00  21s  KeepaliveReceived
```

These observations establish local projection and rendering behavior. They do not establish live topology, HTTP behavior, or a general table-formatting API.
