import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { readFile as readFileAsync } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const licensesDir = join(webRoot, "licenses");
const nodeModules = join(webRoot, "node_modules");

type ClosureEntry = {
  name: string;
  version: string;
  license: string;
  source: string;
};

type ClosureManifest = {
  packages: ClosureEntry[];
};

function fail(message: string): never {
  console.error(`verify-kobalte-licenses: ${message}`);
  process.exit(1);
}

async function sha256File(path: string): Promise<string> {
  const bytes = await readFileAsync(path);
  return createHash("sha256").update(bytes).digest("hex");
}

function packageJsonPath(name: string): string {
  if (name.startsWith("@")) {
    const slashIndex = name.indexOf("/");
    const scope = name.slice(0, slashIndex);
    const packageName = name.slice(slashIndex + 1);
    return join(nodeModules, scope, packageName, "package.json");
  }
  return join(nodeModules, name, "package.json");
}

function readInstalledPackage(name: string): {
  name: string;
  version: string;
  license: string;
} {
  let raw: string;
  try {
    raw = readFileSync(packageJsonPath(name), "utf8");
  } catch {
    fail(`missing installed package ${name} (run bun install --frozen-lockfile)`);
  }
  const parsed = JSON.parse(raw) as {
    name?: string;
    version?: string;
    license?: string;
  };
  if (!parsed.name || !parsed.version || !parsed.license) {
    fail(`incomplete package.json for ${name}`);
  }
  return {
    name: parsed.name,
    version: parsed.version,
    license: parsed.license,
  };
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
  const expected = [...manifest.packages].sort((left, right) =>
    left.name.localeCompare(right.name),
  );
  const seen = new Set<string>();

  for (const entry of expected) {
    if (seen.has(entry.name)) {
      fail(`duplicate inventory entry for ${entry.name}`);
    }
    seen.add(entry.name);

    const installed = readInstalledPackage(entry.name);
    if (installed.version !== entry.version) {
      fail(
        `${entry.name}: inventory version ${entry.version} != installed ${installed.version}`,
      );
    }
    if (installed.license !== entry.license) {
      fail(
        `${entry.name}: inventory license ${entry.license} != installed ${installed.license}`,
      );
    }
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

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  fail(message);
});
