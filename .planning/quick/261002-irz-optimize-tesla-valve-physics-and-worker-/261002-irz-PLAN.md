---
phase: quick-261002-irz
plan: coordinator
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/benchmarks/tesla-valve/README.md
  - README.md
autonomous: true
requirements: [TESLA-PERF-BASELINE, TESLA-PERF-SPATIAL, TESLA-PERF-CACHE-SCRATCH, TESLA-PERF-DRAIN, TESLA-PERF-WORKER, TESLA-PERF-RENDER]
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-261002-irz
generated_at: "2026-10-02T13:32:58-05:00"
must_haves:
  truths:
    - Original immutable benchmark evidence is committed before any optimization edit.
    - All five selected optimizations are complete with exact semantic/quality invariants preserved.
    - Each selected change has an immutable before/after report compared to the immediately preceding accepted stage.
    - Worker playback preserves ordered lifecycle/controls/exports/meter/provenance and renderer caches invalidate correctly.
    - Aggregate source and evidence receive independent exact-digest review before completion.
  artifacts:
    - path: docs/benchmarks/tesla-valve/README.md
      provides: Historical original plus five-stage raw-report/commit index
    - path: web/scripts/bench-tesla.ts
      provides: Reproducible provenance-rich actual production measurement
    - path: web/src/physics/live-session.ts
      provides: Tested worker/direct ownership boundary
  key_links:
    - from: docs/benchmarks/tesla-valve/README.md
      to: docs/benchmarks/tesla-valve/runs
      via: Immutable source/artifact/settings-bound raw reports and previous-stage comparisons
    - from: web/src/player/scene-lifecycle.ts
      to: web/src/physics/worker-session.ts
      via: Tesla-only live worker routing with direct other-scenes/export backend
---

<objective>
Coordinate all five selected Tesla performance improvements with individual attribution and full preserved physics/render quality. This is the quick workflow's entry PLAN; execute the numbered bounded prompts serially through parent-controlled stage gates rather than handing one implementer the entire architecture.
</objective>

<execution-context>
@/Users/peterryszkiewicz/.codex/get-shit-done/workflows/execute-plan.md
@/Users/peterryszkiewicz/.codex/get-shit-done/templates/summary.md
</execution-context>

<context>
@AGENTS.md
@AGENTS.bright-builds.md
@standards-overrides.md
@PROJECT-SCOPE.md
@standards/core/architecture.md
@standards/core/code-shape.md
@standards/core/testing.md
@standards/core/verification.md
@standards/languages/rust.md
@standards/languages/typescript-javascript.md
@.planning/quick/261002-irz-optimize-tesla-valve-physics-and-worker-/261002-irz-CONTEXT.md
@.planning/quick/261002-irz-optimize-tesla-valve-physics-and-worker-/261002-irz-RESEARCH.md

Client date2026-10-02; original clean synced source8c419c4. Standing iteration/push authority and current experimental local macOS scope apply, without package release or mandatory Linux/C++ qualification. Local sidecar/overrides and listed standards materially informed exact semantic ordering, rollback, bounded ownership, test coverage and pre-commit checks. Active global/repository lessons were loaded in full within budgets. Domain-modeling clarified baseline/stage/fingerprint terms; GSD's required locked decisions are recorded in the task CONTEXT.

## Execution and ownership

Parent owns gates, source/report identity, benchmark reports/historical documentation, WASM integration, full verification, commits/state and ordinary push. Agents do not commit. Core stages touch only intended Rust engine paths; no bridge/capture/render optimization in stages1–3. Worker defaults only to Tesla live playback, with direct other-scene/export behavior retained. Preserve concurrent edits. No baseline measured after optimized edits is valid as original.

## Serial dependency graph

| Prompt / wave | Creates | Needs | Selected measurement gate |
| --- | --- | --- | --- |
|01 /1 | Harness and committed original raw reports | Unchanged original implementation | Original baseline |
|02 /2 | Ordered spatial wall candidates |01 accepted baseline | Stage1 vs original |
|03 /3 | Immutable shared polygon/chain geometry |02 accepted stage1 | PartA of stage2; no intermediate accepted stage |
|04 /4 | Transaction-safe reusable scratch |03 | Stage2 vs stage1, both required mechanisms |
|05 /5 | Query-only AABB index without pairs |04 accepted stage2 | Stage3 vs stage2 |
|06 /6 | Worker/direct contracts and ownership |05 accepted stage3 | PartA of stage4 |
|07 /7 | Async lifecycle and frame scheduling |06 | PartB of stage4 |
|08 /8 | Surface/input integration |07 | Stage4 vs stage3, complete worker pipeline |
|09 /9 | Renderer caching/batching and full review |08 accepted stage4 | Stage5 vs stage4 |

Same-wave ownership never overlaps because every prompt is serial. No human checkpoint is planned: the parent checks concrete evidence and continues under standing authority. All selected decisions are full scope; subdivisions preserve one attributed stage2 and one attributed stage4.

## Decision coverage

| Decision | Prompt/task | Coverage |
| --- | --- | --- |
|D-01 baseline/persistence |01 tasks1–3 | Full |
|D-02 spatial contacts |02 tasks1–2 | Full |
|D-03 static cache AND scratch |03 tasks1–2;04 tasks1–3 | Full |
|D-04 drain query |05 tasks1–2 | Full |
|D-05 live Tesla worker |06 tasks1–3;07 tasks1–2;08 tasks1–3 | Full |
|D-06 cache/batch renderer |09 tasks1–3 | Full |
|D-07 five serial gates/parent commits |01 task3;02 task2;04 task3;05 task2;08 task3;09 task3 | Full |
|D-08 exact quality/settings | Every numbered prompt and four-case release/production checks | Full |
|D-09 provenance/noise/components |01 tasks1–2 and each measurement gate | Full |
|D-10 full checks/independent review |09 task3 and parent pre-commit gates | Full |
|D-11 primary vs optional max timing |01 task2 and unchanged per-stage primary suite | Full |
</context>

<tasks>
<task type="auto">
  <name>Task1: Establish original evidence and complete three isolated core stages</name>
  <files>docs/benchmarks/tesla-valve/README.md</files>
  <action>Parent executes bounded prompts01→05 sequentially.01 creates/tests only the harness, then runs/reviews/commits immutable original reports before02 begins.02 spatial contacts gets its own original→stage1 gate.03 shape sharing and04 scratch together get one stage1→stage2 gate; both are required.05 query-only drain gets stage2→stage3 gate. Each parent gate verifies fresh artifact/source identity, primary settings/counts/semantic fingerprints, mandatory four-case quality checks, raw samples/noise and required pre-commit checks; commits the isolated selected stage and evidence before the next selected stage. No bridge/capture/render change may leak into these stages. Repeat/investigate noisy or nonimproving work at its own boundary, never claim unmeasured gains or combine stages to hide attribution.</action>
  <verify><automated>cargo test -p liquidfun &amp;&amp; cargo test -p liquidfun-wasm tesla_valve --release &amp;&amp; (cd web &amp;&amp; bun run test:unit -- tests/bench-tesla.test.ts) &amp;&amp; git diff --check</automated></verify>
  <done>Original plus individually attributed stages1–3 are semantically checked, persistently measured and committed through parent gates.</done>
</task>
<task type="auto">
  <name>Task2: Complete ordered live-worker architecture and stage4 measurement</name>
  <files>docs/benchmarks/tesla-valve/README.md</files>
  <action>Execute06 contracts/ownership,07 lifecycle/scheduling and08 surface/input integration before accepting stage4. Preserve FIFO controls, single in-flight advance, ownership/generation errors, pause/hidden scheduling, direct export/non-Tesla behavior, fixed steps, provenance/meter and all particle settings. Test actual worker/direct same-fixed-step state. Parent compares completed worker stage against stage3 with original direct physics benchmarks plus real live cadence/responsiveness/transport measurements. Label any bounded-batch capture adaptation as stage4 pipeline cost. Validate settings/semantics first, persist raw/uncertainty and parent check/commit gate before renderer edits.</action>
  <verify><automated>just web-build &amp;&amp; (cd web &amp;&amp; bun run typecheck &amp;&amp; bun run test:unit &amp;&amp; bun run test:browser) &amp;&amp; cargo test -p liquidfun-wasm tesla_valve --release</automated></verify>
  <done>All worker quality flows pass and stage4 has individual source-bound actual measurement evidence.</done>
</task>
<task type="auto">
  <name>Task3: Complete renderer stage and independent aggregate closure</name>
  <files>docs/benchmarks/tesla-valve/README.md, README.md</files>
  <action>Execute09 static cache/batching and stable GPU setup with dynamic particle rendering unchanged. Measure stage5 against stage4 using identical primary configuration; preserve all invalidation/appearance/export/fallback checks. Parent produces the historical original+five-stage report index with actual gains/noise/regressions and per-stage commit provenance. Run full required checks in order, preserve failed attempts and review exact scoped aggregate diff. Obtain independent AI acknowledgment from a reviewer who did not implement that diff, bound to actual digest/identity/time. Parent finalizes summary/task/state/ordinary push only with truthful complete evidence. No package release or optional strict qualification is required.</action>
  <verify><automated>cargo fmt --all &amp;&amp; cargo clippy --all-targets --all-features -- -D warnings &amp;&amp; cargo build --all-targets --all-features &amp;&amp; cargo test --all-features &amp;&amp; cargo test -p liquidfun &amp;&amp; cargo test -p liquidfun-wasm --release &amp;&amp; bun scripts/bright-builds-check.ts all &amp;&amp; just markdown-check &amp;&amp; just web-build &amp;&amp; (cd web &amp;&amp; bun run typecheck &amp;&amp; bun run test:unit &amp;&amp; bun run test:browser) &amp;&amp; git diff --check</automated></verify>
  <done>All five locked optimizations and individual measurements are complete, quality checks pass, and independent aggregate review/persistent records support completion.</done>
</task>
</tasks>

<threat-model>
| Boundary | Description |
| --- | --- |
| Spatial/query/cache state to physics | Optimizations must retain exact contact/query order and transactional state. |
| Worker messages to live UI/session | Bounded identity/ownership checks prevent stale or malformed operations. |
| Renderer cache to visible frame | Revision/camera/mode invalidation preserves actual appearance. |
| Timing evidence to persistent claims | Comparable source/artifact/workload identities and uncertainty constrain reported gains. |

| Threat ID | Category | Disposition | Mitigation |
| --- | --- | --- | --- |
|T-PERF-01 |T |mitigate | Exact legacy equivalence/order/rollback tests plus same-backend semantic fingerprints. |
|T-PERF-02 |D/T |mitigate | Bounded worker messages/queue, single in-flight advance, generation guards and disposal tests. |
|T-PERF-03 |T |mitigate | Per-surface exact content/revision keys and dynamic/flip/camera appearance tests. |
|T-PERF-04 |R/T |mitigate | Immutable original+five reports, full provenance/comparability/noise and independent exact-digest review. |
</threat-model>

<verification>
Every selected stage runs its numbered behavior tests, unchanged benchmark primary forward/reverse1440/s suite and mandatory default/max quality guards. Before any parent commit run ordered fmt→strict clippy→all-target build→all-feature tests, relevant native/WASM/web checks, managed/Markdown and diff checks. Original baseline precedes optimization; accepted predecessor reports define every comparison. No claimed speedup or60fps appears before actual evidence.
</verification>

<success-criteria>
All five locked optimizations are fully implemented, exact physical/flow/render quality remains demonstrated, original and five stage comparisons are immutable and individually attributable, and independent aggregate digest review plus complete checks support final completion.
</success-criteria>

<output>
Parent creates .planning/quick/261002-irz-optimize-tesla-valve-physics-and-worker-/261002-irz-SUMMARY.md with run/commit links, actual improvements/noise, verification/reviewer evidence and residual limits. This planning turn edits only quick planning artifacts, not source/STATE/commits/ROADMAP.
</output>


## Execution scheduling clarification

After the committed Stage1 gate, frontend Stage4 preparation may run in a separate managed worktree based on01728fd. Main-checkout integration and all accepted measurements remain serial: Stage2, Stage3, then Stage4, then Stage5. The worker agent must not mutate the measured main checkout or run heavy jobs during canonical timing. Revalidate the complete frontend integration against the accepted Stage3 core before Stage4 measurement. This preserves isolation and attribution while reducing idle preparation time.


Stage5 preparation may also run in the isolated frontend worktree after Stage4 main source is frozen. Only rendering files/tests may change there; main integration waits for the Stage4 accepted evidence commit. No heavy parallel jobs run during timing, and the rendering-only patch is independently applied and revalidated against current main.
