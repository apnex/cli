# Prepare and publish source

The repository is prepared as an experimental Rust source distribution.\
The [preparation review](publication/REVIEW.md) records local checks and remaining findings.\
This guide is a release checklist for the owner; it does not grant permission to publish or make a claim that a release exists.

The owner has selected [repository bootstrap](context/cli-010-bootstrap.json) for `apnex/cli`.\
The [bootstrap record](publication/BOOTSTRAP.md) closes [CLI-010](BACKLOG.md#cli-010) with the delivered private source repository, exact source identity, and passing local and hosted checks.

## Current release boundary

| Surface | Position |
|---|---|
| Source checkout | The initial distribution is the private GitHub source repository, including manifests, lockfiles, declarations, tests, guides, and retained evidence. |
| Project license | [MIT](../LICENSE), selected by the owner in [decision 0002](BACKLOG.md#decision-0002-use-the-mit-license), with matching metadata in both Rust manifests. |
| Hosting destination | [apnex/cli](https://github.com/apnex/cli), private, with `main` as the default branch. |
| Package registry | `publish = false` remains in both [application](../Cargo.toml) and [scaffold](../tools/scaffold/Cargo.toml) manifests. No crates.io package is prepared or claimed. |
| Binary release | Normal-feature builds only; platform coverage is the tested local Linux environment. |
| Automated checks | The [GitHub Actions workflow](../.github/workflows/check.yml) uses read-only repository permissions; the [initial hosted run](https://github.com/apnex/cli/actions/runs/34348866527) passed. Its [run history](https://github.com/apnex/cli/actions/workflows/check.yml) identifies later checked commits. |
| Compatibility | Definition formats, checkpoint declarations, and Rust interfaces remain experimental; retain exact identities when transferring state. |

---

## Review the source contents

Run from the repository root:
```sh
git status --short --untracked-files=all
git ls-files --cached --others --exclude-standard
git diff --check
```

At preparation, only `AGENTS.md` was committed.\
An empty tracked diff therefore does not mean the application is committed or that untracked content is safe to publish.\
Review the full file list, then commit an intentional selection under the owner's release instruction.\
The [verification manifest](evidence/publication/verification.json) identifies the locally tested inputs separately from Git commit identity.

That preparation evidence predates the MIT selection.\
Its license status and source hashes remain historical observations; the [MIT follow-up verification](evidence/publication/license-mit-verification.json) records checks for the subsequent licensing and documentation edits.

The ignore rule excludes Cargo `target` directories at any depth, including the scaffold package's build products.\
Do not use `git clean`, wholesale deletion, or history rewriting as a publication cleanup step.

---

## Retain provenance deliberately

The [source discussion](context/README.md), approvals, acceptance tasks, results, and raw [evidence](evidence/) are part of the engineering record.\
The preparation preserves them intact instead of making historical outputs match a new machine or toolchain.\
Their machine paths are historical provenance, not portable prerequisites.

Before making the repository public, the owner reviews that retained material for intended disclosure.\
A bounded credential-pattern search is evidence about its named patterns, not a guarantee that every sensitive value has been identified.\
If a record requires withholding, preserve its original privately and document the public omission explicitly; do not silently rewrite a failure, approval, or task.

The [upstream JSON Schema fixtures](constraints/acceptance/upstream/README.md) retain their [license](constraints/acceptance/upstream/LICENSE) and provenance.\
Dependency license metadata is recorded in the preparation evidence.\
Retain the project MIT notice and the existing third-party notices when assembling a distribution.

---

## Verify a release candidate

Complete the [contributor verification](CONTRIBUTING.md#verification), then execute [getting started](GETTING-STARTED.md) from a clean copy containing only the intended source files.\
Use the same declared toolchain and lockfiles.\
Check direct commands, simulation labels, reopening a named session, and optional Cargo installation and removal.

A source build needs the operation declaration files under `docs/` because some are embedded during compilation.\
Do not exclude `docs/` wholesale from a package or source archive.\
A runtime-only binary consumer can use an exported spec without the source checkout; authoring and developer workflows need their documented inputs.

Record the exact commit after the selected files are committed, the build environment, the normal executable digest, and the raw test results.\
Preparation evidence cannot certify later edits or a different release candidate.\
The strict Clippy baseline and any advisory-scan limits remain visible in the [review](publication/REVIEW.md).

---

## Owner decisions before publication

| Decision | Required result |
|---|---|
| License | Resolved: the owner selected [MIT](../LICENSE); both Rust manifests declare `license = "MIT"`. |
| Destination and visibility | Destination resolved: `apnex/cli`; bootstrapped as private using the stated default. Public visibility remains an owner choice. |
| Historical disclosure | Retain the reviewed source discussion, approval records, paths, and test evidence in the bootstrap; public disclosure follows the owner's visibility selection. |
| Release scope | Source repository for this bootstrap; package or binary distribution remains a separate selection. |
| Release authorization | The owner's bootstrap instruction authorized the completed initial source push and verification; it does not authorize a separate package or binary release. |

The [board](BOARD.md) closes the source bootstrap and retains broader distribution choices.\
A published repository or successful upload does not establish production readiness or universal CLI coverage.

---

## Mechanics, rationale, and consequence

### Mechanics

Review intended files, verify a clean source consumer, resolve owner-held distribution decisions, and identify the exact published revision and artifacts.

### Rationale

A reproducible local application becomes a usable source release only when its contents and consumer contract are deliberate.

### Consequence of violation

A release can omit required declarations, publish unintended records, imply unchosen licensing terms, or claim tests for bytes that were never tested.
