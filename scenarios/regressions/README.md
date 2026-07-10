# Differential Regression Format

This directory accepts only explicitly reviewed, minimized semantic regressions. A regression is
the exact canonical JSON serialization of the validated scenario value that reproduced a physics
mismatch. A seed alone is not a regression because generator implementations and strategies may
change.

Promotion metadata must retain all of the following evidence:

- the exact serialized scenario content and its SHA-256;
- the original named or seeded source, including generator ID, generator version, and seed when
  applicable;
- the complete first-divergence failure signature: checkpoint ID, named phase, typed semantic path,
  and mismatch kind;
- protocol, scenario, trace, and tolerance-profile versions and hashes;
- oracle, adapter, compiler, target, flags, notice, generator-revision, and reviewer identity.

Generation writes candidates only below `target/differential/staging`. Review reparses and replays
the candidate, proves the same failure signature, and renders a deterministic diff. Only the
explicit promotion command may add a reviewed file here; tests, checks, and replay commands are
read-only. Phase 2 uses synthetic mismatch coverage and does not claim that a real LiquidFun
physics mismatch has been discovered.
