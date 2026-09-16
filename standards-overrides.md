# Standards Overrides

Use this file to record deliberate deviations from the canonical coding and architecture standards.

## Active overrides

| Standard                | Local decision                | Rationale             | Owner                | Review date         |
| ----------------------- | ----------------------------- | --------------------- | -------------------- | ------------------- |
| `REPLACE_WITH_STANDARD` | `REPLACE_WITH_LOCAL_DECISION` | `REPLACE_WITH_REASON` | `REPLACE_WITH_OWNER` | `REPLACE_WITH_DATE` |

## Notes

### Autonomous iteration authorization — 2026-09-13

The repository owner's standing authorization is defined in `AGENTS.md` under `Standing authorization for autonomous iteration`. It replaces older per-attempt human approval requirements for implementation, corrective verification, ordinary main-branch publication, and relevant validation workflows. Apply it when a plan, skill, or historical failure record would otherwise stop routine recovery for a new approval token. Verification requirements and truthful evidence remain mandatory.

- Prefer narrow, explicit exceptions over broad "this repo is different" statements.
- If local verification is intentionally hook-owned or leaves heavy suites to CI, record that explicitly here.
- Revisit overrides periodically instead of letting them become permanent by accident.
- If an override becomes common across many repos, move it back upstream into the canonical standards repo.

### Independent AI review — 2026-09-16

Apply the `Independent review` policy in `AGENTS.md` when older plans or records require a human reviewer. The owner permits an identified independent AI reviewer; all other evidence and acknowledgment requirements remain in force.
