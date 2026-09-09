# Continue the project

Read the [board](BOARD.md) for current work and the [backlog](BACKLOG.md) for the evidence and disposition behind each record.\
The [documentation index](README.md) routes by workflow, so recovering the whole conversation is unnecessary for ordinary use or development.

## Recover the intent

Read the [vision](../VISION.md) and [captured intent](project-intent.md).\
Consult the [original discussion](context/README.md) when an interpretation or correction matters.\
The owner's emphasis is contextual construction, reuse, portability, deterministic interpretation, and another actor's ability to resume.\
A substantial specification can be valuable; specification size alone does not settle workflow value.

A definition describes intended system structure.\
Its mock results and historical observations do not establish the target system's current state.

---

## Inspect the scaffold

The [architecture](ARCHITECTURE.md) is a target responsibility model with explicit section maturity.\
The [layer registry](layers.json) owns generated responsibility views; those documentation scopes are not separate Rust crates or certified subsystems.\
The [source map](CONTRIBUTING.md#source-map) identifies the implementation, and the [contributor workflow](CONTRIBUTING.md#verification) gives its repeatable checks.

The current local workflows include JSON authoring, definition construction, schema constraints, component assembly, explicit JSON read grants, and direct run mode.\
Use their guides and contracts through the [workflow index](README.md#choose-a-workflow).\
Do not infer their current verification status from the existence of a source file or historical test output.

---

## Choose concrete work from the open questions

Continue work the owner has already authorized without asking for that authorization again.\
Held items carry observable triggers and return to triage when their requirements exist.\
The [architecture's open register](ARCHITECTURE.md#owed-and-open-register) identifies broader capability consumers without selecting their implementation.

The [first authoring decision](authoring/CLI-001.md#decision-0001-establish-the-bounded-authoring-contract) fixes the original session contract.\
Later records retain their own approvals and bounded acceptance tasks.\
Read the relevant original task before extending a test or oracle; preserve earlier failures even when a later method succeeds.\
The [publication review](publication/REVIEW.md) distinguishes repository preparation from actual release and from independent assurance.

---

## Resume artifacts correctly

| Artifact | What to preserve or recover |
|---|---|
| Session checkpoint | Exact compatible kernel declaration, original intent, candidate, accepted baseline, revision, and latest receipt. |
| Active interface export | Definition, runtime state, provenance, and historical invocation; current capability grants are supplied separately. |
| Component sources | Identity, dependencies, namespaces, state ownership, and embedded source provenance. |
| Fresh-agent experiment | Original task, evaluator, input package, actor exposure, and measured outcome. |
| Publication candidate | Complete intended source contents, exact revision or file manifest, environment, and raw checks. |

Saved checkpoints must be reopened with the matching declaration bytes; a new declaration is not a silent receipt migration.\
A fresh run can consume a portable interface export instead.\
For a named run session, `-` resumes without rereading the original spec; see the [run guide](run/USE.md).\
Read grants are process authority and must be supplied again for fresh connected observations.

The recorded [fresh-agent package](reuse/FRESH-ACTOR-TRIAL.md) contains completed work.\
Prepare a new package through the [reuse guide](reuse/USE.md) for another trial rather than calling that package untouched.\
The [comparison results](authoring/IMPORT-RESULTS.md) distinguish scripted workflow costs from unmeasured comparative agent effort.

---

## Corrections retained

Earlier continuation guidance said "Reusable component resolution and broader agent continuation remain later board work" after the selected local assembly and fresh-agent scopes were already completed.\
That stale statement is withdrawn.\
The [component results](components/RESULTS.md) and [fresh-agent trial](reuse/FRESH-ACTOR-TRIAL.md) establish separate bounded observations; broader package resolution and comparative agent savings remain open.

Historical toolchain paths and binary locations identify the original test environment.\
They are not requirements for a new source checkout.\
Build and run the current documented checks instead of copying old machine paths into a new workflow.

---

## Mechanics, rationale, and consequence

### Mechanics

Follow intent to the selected record, read the owning contract, preserve its original evidence, and measure the current source before making a new claim.

### Rationale

A fresh actor can continue useful work without reconstructing hidden decisions or treating old observations as fresh state.

### Consequence of violation

A continuation can reopen the wrong checkpoint, repeat an already completed increment, or erase the distinction between simulation and a granted observation.
