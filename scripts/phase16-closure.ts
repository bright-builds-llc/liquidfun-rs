import { relative, resolve } from "node:path";

import {
  runClosureAttempt,
  type CommandSpec,
} from "./phase16/closure-runner";

const commands: readonly CommandSpec[] = [
  {
    id: "closure-regressions",
    argv: ["bun", "test", "scripts/phase16"],
  },
  {
    id: "cargo-fmt",
    argv: ["cargo", "fmt", "--all", "--check"],
  },
  {
    id: "wasm-tests",
    argv: ["cargo", "test", "-p", "liquidfun-wasm"],
  },
  {
    id: "wasm-clippy",
    argv: [
      "cargo",
      "clippy",
      "-p",
      "liquidfun-wasm",
      "--all-targets",
      "--",
      "-D",
      "warnings",
    ],
  },
  {
    id: "web-build",
    argv: ["just", "web-build"],
  },
  {
    id: "native-build",
    argv: ["cargo", "build", "-p", "liquidfun"],
  },
  {
    id: "native-tests",
    argv: [
      "cargo",
      "test",
      "-p",
      "liquidfun",
      "--all-features",
    ],
  },
  {
    id: "default-build",
    argv: ["cargo", "build"],
  },
  {
    id: "package-verify",
    argv: ["cargo", "xtask", "package", "verify"],
  },
  {
    id: "aggregate-check",
    argv: ["just", "check"],
  },
  {
    id: "markdown-check",
    argv: ["just", "markdown-check"],
  },
  {
    id: "bright-builds",
    argv: ["bun", "scripts/bright-builds-check.ts", "all"],
  },
  {
    id: "git-diff-check",
    argv: ["git", "diff", "--check"],
  },
  {
    id: "native-tree",
    argv: ["cargo", "tree", "-p", "liquidfun", "--edges", "normal"],
  },
  {
    id: "cargo-metadata",
    argv: ["cargo", "metadata", "--format-version", "1", "--no-deps"],
  },
  {
    id: "package-list",
    argv: [
      "cargo",
      "package",
      "-p",
      "liquidfun",
      "--list",
      "--allow-dirty",
    ],
  },
];

const repoRoot = resolve(import.meta.dir, "..");

function requireAttemptPath(maybePath: string | undefined): string {
  if (maybePath === undefined) {
    throw new Error(
      "usage: bun scripts/phase16-closure.ts target/phase16/closure-attempt-N",
    );
  }
  const path = resolve(repoRoot, maybePath);
  const relativePath = relative(repoRoot, path);
  if (!/^target\/phase16\/closure-attempt-\d+$/.test(relativePath)) {
    throw new Error(`invalid Phase 16 closure attempt: ${relativePath}`);
  }
  return path;
}

await runClosureAttempt(
  repoRoot,
  requireAttemptPath(process.argv[2]),
  commands,
);
