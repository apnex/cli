# Native application design audit

Verdict: pass-with-guardrails before implementation, engineering self-audit under mission-kit M7.
The owner's selected full integration fixes direction; implementation choices do not require a survey.

| Anchor | Constraint and falsifier |
|---|---|
| A1 transparency | Help exposes declared capabilities and granted state; errors and historical receipts remain distinguishable from fresh reads. |
| A2 specification | Embedded data drives all AGP verbs and views; a command implemented only in AGP-specific parser code fails the boundary. |
| A3 composition | Generic HTTP, launcher, navigation, and tables remain in CLI; management vocabulary remains in AGP. |
| A4 fidelity | Existing connection/route oracles and response data remain intact; frozen parity is compared without weakening assertions. |
| A8 integrity | Validate profile before dispatch, reject ambiguous aliases, bound HTTP, and require actual process statuses. |
| A9 failure validation | Exercise denied targets, redirects, HTTP errors, malformed data, and missing authority; no deployment certification is claimed. |
| A13 intent | Complete the selected local integration without requesting repeated authorization. |

Inputs read: AGP shell dispatcher/drivers/templates, management server and response schemas, local e2e parity and process harness, CLI run/receipt/view implementation, and official ureq configuration documentation.
These are mechanism and compatibility inputs, not an independent reviewer verdict.
The user explicitly selects shared CLI functionality for the first consumer; recurrence across two consumers remains unmeasured.

Guardrails: preserve legacy serialized file receipts, keep launch authority transient, share all frontend dispatch, retain original failures and evidence, run both projects' required checks, and test literal operator instructions.

## Implementation disposition

The [measured results](RESULTS.md) satisfy local acceptance with successful IPv6 transport explicitly unmeasured on this host.
Source review compared the shared run path, original AGP behavior and schema contracts, owner intent, and [ureq's documented configuration](https://docs.rs/ureq/latest/ureq/config/struct.Config.html).
This remains an engineering self-audit; no independent reviewer verdict or hosted deployment is claimed.
