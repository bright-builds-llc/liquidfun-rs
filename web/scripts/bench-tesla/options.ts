import type { BenchmarkStage, RunRole } from "./contracts";

export type BenchmarkOptions = {
  readonly help: boolean; readonly stage: BenchmarkStage; readonly role: RunRole;
  readonly maybeRunId: string | undefined; readonly maybeOutputRoot: string | undefined;
  readonly maybePrevious: string | undefined;
  readonly browserChannel: "chromium" | "headless-shell";
};

export function parseOptions(args: readonly string[]): BenchmarkOptions {
  const flags = args.filter((value) => value !== "--");
  let stage: BenchmarkStage = "original";
  let maybeRole: RunRole | undefined;
  let maybeRunId: string | undefined, maybeOutputRoot: string | undefined, maybePrevious: string | undefined;
  let help = false;
  let browserChannel: "chromium" | "headless-shell" = "chromium";
  for (let index = 0; index < flags.length; index += 1) {
    const flag = flags[index];
    if (flag === "--help" || flag === "-h") { help = true; continue; }
    if (flag === "--smoke") { stage = "smoke"; continue; }
    if (!["--stage", "--role", "--run-id", "--output-root", "--previous", "--browser-channel"].includes(flag ?? "")) throw new Error(`Unknown benchmark option ${flag}`);
    const value = flags[++index];
    if (value === undefined || value.startsWith("--")) throw new Error(`Missing value for ${flag}`);
    if (flag === "--stage") {
      if (!["original", "stage1", "stage2", "stage3", "stage4", "stage5"].includes(value)) throw new Error("Unknown benchmark stage");
      stage = value as BenchmarkStage;
    }
    if (flag === "--role") {
      if (!["before", "after"].includes(value)) throw new Error("Role must be before or after");
      maybeRole = value as RunRole;
    }
    if (flag === "--run-id") maybeRunId = value;
    if (flag === "--output-root") maybeOutputRoot = value;
    if (flag === "--previous") maybePrevious = value;
    if (flag === "--browser-channel") {
      if (value !== "chromium" && value !== "headless-shell") throw new Error("Browser channel must be chromium or headless-shell");
      browserChannel = value;
    }
  }
  const role = stage === "original" ? "baseline" : stage === "smoke" ? "smoke" : maybeRole ?? "after";
  if (!help && (stage === "original" || stage === "smoke") && (maybeRole !== undefined || maybePrevious !== undefined)) throw new Error("Original/smoke runs cannot link a previous stage or specify a role");
  if (!help && stage !== "original" && stage !== "smoke" && maybePrevious === undefined) throw new Error("An optimization stage requires --previous report.json");
  return { help, stage, role, maybeRunId, maybeOutputRoot, maybePrevious, browserChannel };
}

export const HELP = `Usage: bun run bench:tesla -- [options]

  --stage original|stage1|stage2|stage3|stage4|stage5  (default: original)
  --role before|after       Fresh before/after record for an optimization stage
  --previous REPORT.json   Immediate preceding accepted or same-stage before report
  --run-id ID              Fresh lowercase run ID; existing runs are never overwritten
  --output-root DIRECTORY  Default: docs/benchmarks/tesla-valve/runs
  --smoke                  Small non-comparable harness check (not baseline evidence)
  --browser-channel chromium|headless-shell  Default: chromium (new full headless)
  --help                   Show this help

Canonical profile: forward warm360, reverse warm384, 40 one-step samples,
3 sequential repeats, rate1440, dt1/60, particle4/velocity8/position3 iterations,
5mm radius, gravity(0,-10), inlet2m/s, damping0.2/friction0.05, 1280x960/DPR1,
shaded-blob/all particles, plus actual production RAF cadence for >=2s.
Renderer timing is CPU submission, not GPU completion; FPS is observed cadence,
not reciprocal CPU time. Live trajectories advance different distances across stages.
Every run rebuilds production WASM and an isolated Vite benchmark bundle.
`;
