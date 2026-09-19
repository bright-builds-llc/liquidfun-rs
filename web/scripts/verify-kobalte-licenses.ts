import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { readFile as readFileAsync } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const webRoot = join(dirname(scriptPath), "..");
const licensesDir = join(webRoot, "licenses");
const nodeModules = join(webRoot, "node_modules");
const ROOT_PACKAGE_NAME = "@kobalte/core";

export type ClosureEntry = {
  name: string;
  version: string;
  license: string;
  source: string;
};

type ClosureManifest = {
  root: string;
  packages: ClosureEntry[];
};

export type InstalledPackage = ClosureEntry & {
  directory: string;
  dependencies: Readonly<Record<string, string>>;
};

type PackageReader = (
  name: string,
  maybeParentDirectory?: string,
) => InstalledPackage;

function fail(message: string): never {
  console.error(`verify-kobalte-licenses: ${message}`);
  process.exit(1);
}

async function sha256File(path: string): Promise<string> {
  const bytes = await readFileAsync(path);
  return createHash("sha256").update(bytes).digest("hex");
}

function packagePathParts(name: string): string[] {
  if (name.startsWith("@")) {
    const slashIndex = name.indexOf("/");
    const scope = name.slice(0, slashIndex);
    const packageName = name.slice(slashIndex + 1);
    return [scope, packageName];
  }
  return [name];
}

function resolvePackageJson(
  name: string,
  maybeParentDirectory?: string,
): string {
  const packageParts = packagePathParts(name);
  let currentDirectory = maybeParentDirectory ?? webRoot;

  while (currentDirectory.startsWith(webRoot)) {
    const candidate = join(
      currentDirectory,
      "node_modules",
      ...packageParts,
      "package.json",
    );
    if (existsSync(candidate)) {
      return candidate;
    }

    if (currentDirectory === webRoot) {
      break;
    }
    currentDirectory = dirname(currentDirectory);
  }

  const rootCandidate = join(nodeModules, ...packageParts, "package.json");
  if (existsSync(rootCandidate)) {
    return rootCandidate;
  }

  throw new Error(
    `missing installed package ${name} (run bun install --frozen-lockfile)`,
  );
}

function normalizeSource(source: string): string {
  return source
    .trim()
    .replace(/^git\+/, "")
    .replace(/^git:\/\/github\.com\//, "https://github.com/")
    .replace(/^ssh:\/\/git@github\.com\//, "https://github.com/")
    .replace(/\.git$/, "")
    .replace(/\/$/, "");
}

function repositorySource(
  name: string,
  repository: string | { url?: string } | undefined,
): string {
  const maybeSource =
    typeof repository === "string" ? repository : repository?.url;
  if (maybeSource === undefined) {
    throw new Error(`missing repository source for installed package ${name}`);
  }
  return normalizeSource(maybeSource);
}

function readInstalledPackage(
  name: string,
  maybeParentDirectory?: string,
): InstalledPackage {
  const packageJson = resolvePackageJson(name, maybeParentDirectory);
  const raw = readFileSync(packageJson, "utf8");
  const parsed = JSON.parse(raw) as {
    name?: string;
    version?: string;
    license?: string;
    repository?: string | { url?: string };
    dependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
  };
  if (!parsed.name || !parsed.version || !parsed.license) {
    throw new Error(`incomplete package.json for ${name}`);
  }
  if (parsed.name !== name) {
    throw new Error(
      `resolved ${name} to package metadata for ${parsed.name}`,
    );
  }

  return {
    name: parsed.name,
    version: parsed.version,
    license: parsed.license,
    source: repositorySource(name, parsed.repository),
    directory: dirname(packageJson),
    dependencies: {
      ...parsed.dependencies,
      ...parsed.optionalDependencies,
    },
  };
}

export function deriveRuntimeClosure(
  rootName: string,
  readPackage: PackageReader,
): InstalledPackage[] {
  const pending: Array<{
    name: string;
    maybeParentDirectory?: string;
  }> = [{ name: rootName }];
  const packages = new Map<string, InstalledPackage>();

  while (pending.length > 0) {
    const next = pending.shift();
    if (next === undefined) {
      break;
    }

    const installed = readPackage(next.name, next.maybeParentDirectory);
    const maybeExisting = packages.get(installed.name);
    if (maybeExisting !== undefined) {
      if (maybeExisting.version !== installed.version) {
        throw new Error(
          `multiple installed versions of ${installed.name} are reachable: ` +
            `${maybeExisting.version} and ${installed.version}`,
        );
      }
      continue;
    }

    packages.set(installed.name, installed);
    for (const dependencyName of Object.keys(installed.dependencies).sort()) {
      pending.push({
        name: dependencyName,
        maybeParentDirectory: installed.directory,
      });
    }
  }

  return [...packages.values()].sort((left, right) =>
    left.name.localeCompare(right.name),
  );
}

export function inventoryDifferences(
  derived: readonly InstalledPackage[],
  tracked: readonly ClosureEntry[],
): string[] {
  const differences: string[] = [];
  const derivedByName = new Map(derived.map((entry) => [entry.name, entry]));
  const trackedByName = new Map<string, ClosureEntry>();

  for (const entry of tracked) {
    if (trackedByName.has(entry.name)) {
      differences.push(`duplicate tracked entry: ${entry.name}`);
      continue;
    }
    trackedByName.set(entry.name, entry);
  }

  for (const entry of derived) {
    const maybeTracked = trackedByName.get(entry.name);
    if (maybeTracked === undefined) {
      differences.push(`missing tracked entry: ${entry.name}`);
      continue;
    }

    if (maybeTracked.version !== entry.version) {
      differences.push(
        `${entry.name}: tracked version ${maybeTracked.version} != installed ${entry.version}`,
      );
    }
    if (maybeTracked.license !== entry.license) {
      differences.push(
        `${entry.name}: tracked license ${maybeTracked.license} != installed ${entry.license}`,
      );
    }
    if (normalizeSource(maybeTracked.source) !== entry.source) {
      differences.push(
        `${entry.name}: tracked source ${maybeTracked.source} != installed ${entry.source}`,
      );
    }
  }

  for (const entry of tracked) {
    if (!derivedByName.has(entry.name)) {
      differences.push(`extra tracked entry: ${entry.name}`);
    }
  }

  return differences;
}

async function verifyPreservedUpstreamFiles(): Promise<void> {
  const noticeInstalled = join(nodeModules, "@kobalte/core/NOTICE.txt");
  const noticePreserved = join(licensesDir, "kobalte-core-0.13.12-NOTICE.txt");
  const mitInstalled = join(nodeModules, "@kobalte/core/LICENSE.md");
  const mitPreserved = join(licensesDir, "kobalte-core-0.13.12-MIT.txt");

  for (const path of [noticeInstalled, noticePreserved, mitInstalled, mitPreserved]) {
    try {
      await readFileAsync(path);
    } catch {
      fail(`missing file ${path}`);
    }
  }

  const [
    noticeInstalledHash,
    noticePreservedHash,
    mitInstalledHash,
    mitPreservedHash,
  ] = await Promise.all([
    sha256File(noticeInstalled),
    sha256File(noticePreserved),
    sha256File(mitInstalled),
    sha256File(mitPreserved),
  ]);

  if (noticeInstalledHash !== noticePreservedHash) {
    fail("kobalte-core-0.13.12-NOTICE.txt is not byte-identical to installed NOTICE.txt");
  }
  if (mitInstalledHash !== mitPreservedHash) {
    fail("kobalte-core-0.13.12-MIT.txt is not byte-identical to installed LICENSE.md");
  }
}

function verifyClosureInventory(manifest: ClosureManifest): void {
  const derived = deriveRuntimeClosure(
    ROOT_PACKAGE_NAME,
    readInstalledPackage,
  );
  const root = derived.find((entry) => entry.name === ROOT_PACKAGE_NAME);
  if (root === undefined) {
    throw new Error(`derived closure does not include ${ROOT_PACKAGE_NAME}`);
  }
  if (manifest.root !== `${root.name}@${root.version}`) {
    throw new Error(
      `tracked root ${manifest.root} != installed ${root.name}@${root.version}`,
    );
  }

  const differences = inventoryDifferences(derived, manifest.packages);
  if (differences.length > 0) {
    throw new Error(`runtime closure mismatch: ${differences.join("; ")}`);
  }
}

async function main(): Promise<void> {
  const manifestRaw = await readFileAsync(
    join(licensesDir, "kobalte-runtime-closure.json"),
    "utf8",
  );
  const manifest = JSON.parse(manifestRaw) as ClosureManifest;

  await verifyPreservedUpstreamFiles();
  verifyClosureInventory(manifest);

  console.log(
    `verify-kobalte-licenses: ok (${manifest.packages.length} packages, preserved NOTICE and MIT)`,
  );
}

if (process.argv[1] === scriptPath) {
  main().catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    fail(message);
  });
}
