# GitHub repository bootstrap

**Status: private source repository bootstrapped; local and hosted verification passed.**\
This is the delivery record for [CLI-010](../BACKLOG.md#cli-010).\
The owner's instruction was "Bootstrap the apnex/cli remote"; the [source statement](../context/cli-010-bootstrap.json) retains the instruction and its interpretation.

## Delivered source

| Surface | Measured result |
|---|---|
| Repository | [apnex/cli](https://github.com/apnex/cli), created under the authenticated owner's account. |
| Visibility | Private; this was the stated implementation default while the optional visibility question remained unanswered. No owner instruction to make the source public is inferred. |
| Branch | `main`, with local `origin` pointing to the GitHub repository and the local branch tracking `origin/main`. |
| Initial source revision | `c4e060feaf82b2d2af15f9b1edde133ba799018c`. |
| Initial source tree | `51c14205877e08a2d29e7b49d974398d192939fd`, containing 5,040 files. |
| License | GitHub recognizes MIT; the root license and both Rust package declarations are included. |
| Distribution | Source repository; Cargo registry publishing remains disabled and no binary release or release tag was created. |

The [remote branch response](../evidence/remote-bootstrap/initial-remote-branch.json) identifies the same commit and tree as the local source candidate.\
The [push output](../evidence/remote-bootstrap/initial-push.log) records the initial branch transfer.\
Cloning back from GitHub reproduced that commit and tree, and the clone passed Git's integrity check.\
These are source-delivery observations, not evidence that a deployed application or an external service is running.

---

## Verification

The candidate was tested from a clean local clone of the committed source, using the declared toolchain and lockfiles.\
The [verification record](../evidence/remote-bootstrap/verification.json) identifies the exact commit, tree, normal executable digest, commands, environment, and local and hosted checks.

| Check | Result |
|---|---|
| Normal application suite | 70 passed. |
| Instrumented application suite | 87 passed, including the continuation and recovery experiments. |
| Documentation suite | 13 passed, including board/backlog correspondence and local links. |
| Formatting and generated views | Both Rust packages pass formatting; seven layers and 15 generated views remain synchronized. |
| Normal release build | Passed after the instrumented suite. |
| Literal user journey | Six main shell blocks from [getting started](../GETTING-STARTED.md) passed through authoring, export, direct invocation, and session reopening. |
| Hosted verification | [Initial GitHub Actions run](https://github.com/apnex/cli/actions/runs/34348866527) passed for the source revision above; the [job result](../evidence/remote-bootstrap/hosted-ci.json) and [raw log](../evidence/remote-bootstrap/hosted-ci.log) are retained. |

The repeated local journey did not repeat optional installation and removal; those results remain in the earlier [preparation verification](../evidence/publication/verification.json).\
Local execution and CI remain checks by the implementation's own tests; independent behavioral assurance and broader platform support are not claimed.

This record and the board closeout follow the verified source revision as documentation and evidence changes.\
The [GitHub Actions history](https://github.com/apnex/cli/actions/workflows/check.yml) identifies checks for later commits.

---

## Preservation and review findings

The [preservation comparison](../evidence/remote-bootstrap/preservation-review.json) found 62 unchanged code and test files and 4,857 unchanged historical, context, and upstream files.\
The bootstrap added repository metadata and current release records; it changed no runtime implementation or acceptance task.\
The [bounded credential-pattern scan](../evidence/remote-bootstrap/credential-pattern-review.json) examined 5,039 initial candidate files and found no matches for its five named pattern families.

The initial full staged whitespace check returned exit 2 with [307 findings](../evidence/remote-bootstrap/initial-staged-whitespace.log): two in upstream fixtures and 305 in retained evidence.\
The same check excluding those historical inputs and the source-context directory returned exit 0, with no findings outside upstream fixtures and retained evidence.\
The original bytes are preserved; this record does not turn the full staged check into a passing result.

The earlier [publication review](REVIEW.md) and MIT-selection evidence retain their original source identities and observations.\
The existing lint and architecture-projection findings remain held on the [board](../BOARD.md); repository delivery does not resolve them.
