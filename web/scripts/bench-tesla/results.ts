import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

/** Reserves a fresh evidence directory; retries can never replace an earlier run. */
export async function reserveRunDirectory(root: string, runId: string): Promise<string> {
  if (!/^[a-z0-9][a-z0-9-]{0,119}$/.test(runId)) throw new Error("Invalid benchmark run ID");
  await mkdir(root, { recursive: true });
  const directory = join(root, runId);
  await mkdir(directory);
  return directory;
}

export async function writeImmutableJson(file: string, value: unknown): Promise<void> {
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });
}
