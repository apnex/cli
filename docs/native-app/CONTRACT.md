# Native configured applications

The owner selected full local AGP integration in `agp/rustcli` as the first consumer.
The existing CLI runtime owns dispatch, navigation, observation receipts, and presentation.
AGP owns its management paths, response requirements, verbs, views, and operator defaults as configuration.

## Selected transition

Add an embeddable application launcher and a bounded native HTTP JSON GET provider.
An application profile declares its endpoint option/environment, exact capability-to-path grants, input requirements, control aliases, context help preference, output default, and error status mapping.
The profile contains no executable snippets.
The application entry point embeds the definition and profile and calls the shared launcher.

For this consumer, cover all ten resources declared by AGP's management server.
Retain `connections.list` and `routes.list`; offer hierarchical contexts with declared `show` and `ls` verbs.
Default output is a native table; `--json` emits the management response and `--events` emits runtime receipts.
Interactive navigation supports `up`, `top`, `?`, `help`, `tree`, and `exit`, with the canonical colon controls retained.
Reject control aliases that shadow configured routes.

HTTP grants are transient authority, not reconstructed from definitions, exports, or receipts.
Accept only explicit HTTP literal loopback addresses with a valid port and fixed absolute resource paths.
Disable proxies and redirects; use two-second connection and seven-second overall deadlines.
Require status 200 and bounded UTF-8 JSON, at most one MiB.
Evaluate the configured response requirement even for raw JSON output.
Persist byte and exact-output identities without persisting endpoint grants.
Failed HTTP reads and invalid responses preserve the previous invocation.
Presentation failure retains the successful read receipt.

## Completion predicates

1. Every AGP resource is reachable by declared contextual and dotted verbs and has a useful native view.
2. The specification can be reconstructed through the contextual authoring CLI and reused by the embedded launcher.
3. Existing AGP frozen parity, live timing, independent-process, and loopback inspection tests run against the Rust executable.
4. Additional live tests cover all management resources, aliases, help, navigation, error boundaries, and execution without shell helpers.
5. CLI's existing authoring, transfer, recovery, output, and lint gates remain satisfied.
6. Documentation records commands actually executed and separates local live-suite observations from deployment claims.

This is a read-only management consumer; the management API supplies no mutation verbs.
Dynamic entity contexts and general authenticated remote HTTP are outside this selected consumer.
No universal-consumer or independent-review claim follows from one consumer's passing tests.
