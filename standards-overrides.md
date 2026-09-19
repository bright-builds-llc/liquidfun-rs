# Standards Overrides

Use this file to record deliberate deviations from the canonical coding and architecture standards.

## Active overrides

| Standard                                                                 | Local decision                                                                       | Rationale                                                                                                                                                                                | Owner            | Review date |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ----------- |
| `standards/languages/typescript-javascript.md` MysticUI+Tailwind default | Semantic HTML and scoped CSS with Kobalte Dialog for the responsive playground shell | The owner replaced the Phase 17 thin-slice no-library decision; Kobalte supplies accessible modal behavior without a broad design-system migration                                       | Repository owner | 2026-12-17  |
| TypeScript dependency declaration checking                               | Keep `skipLibCheck: true` in `web/tsconfig.json`                                     | Kobalte 0.13.12 generated declarations trigger TS2693 under pinned TypeScript 7.0.2; revisit and remove this override when Kobalte's declarations support the pinned TypeScript compiler | Repository owner | 2026-12-17  |

## Notes

### Autonomous iteration authorization — 2026-09-13

The repository owner's standing authorization is defined in `AGENTS.md` under `Standing authorization for autonomous iteration`. It replaces older per-attempt human approval requirements for implementation, corrective verification, ordinary main-branch publication, and relevant validation workflows. Apply it when a plan, skill, or historical failure record would otherwise stop routine recovery for a new approval token. Verification requirements and truthful evidence remain mandatory.

- Prefer narrow, explicit exceptions over broad "this repo is different" statements.
- If local verification is intentionally hook-owned or leaves heavy suites to CI, record that explicitly here.
- Revisit overrides periodically instead of letting them become permanent by accident.
- If an override becomes common across many repos, move it back upstream into the canonical standards repo.

### Independent AI review — 2026-09-16

Apply the `Independent review` policy in `AGENTS.md` when older plans or records require a human reviewer. The owner permits an identified independent AI reviewer; all other evidence and acknowledgment requirements remain in force.

### Hobby-project scope — 2026-09-16

`PROJECT-SCOPE.md` records the owner's revised goals and overrides older mandatory Linux x64 and dedicated-runner completion requirements. Strict certification tools keep their existing evidence semantics when explicitly invoked. Ordinary local Rust checks, licensing, honest claims and relevant regressions remain required. Ordinary Cargo CI uses one macOS job; expensive validation is manual-only. Apply the accepted experimental preparation checklist, best-effort non-macOS support, incremental parity and evolving API policy. Durable API/MSRV guarantees are deferred; the declared minimum compiler still requires verification before publication.
