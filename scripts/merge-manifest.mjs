#!/usr/bin/env node
// Merges per-file manifest JSONs written by the plugin's manifestDir option into a
// single manifest file.
//
// Usage:
//   node scripts/merge-manifest.mjs <manifestDir> [outputFile]
//
// <manifestDir>  Directory containing *.json files written by the plugin.
// [outputFile]   Path for the merged output. Defaults to testid-manifest.json.
//
// Output format:
//   { "src/Foo.tsx": ["Foo.div", "Foo.button"], "src/Bar.tsx": ["Bar.span"] }

import { readdir, readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

const [, , manifestDir, outputFile = "testid-manifest.json"] = process.argv;

if (!manifestDir) {
  console.error(
    "Usage: node scripts/merge-manifest.mjs <manifestDir> [outputFile]"
  );
  process.exit(1);
}

const dir = resolve(manifestDir);
let entries;
try {
  entries = await readdir(dir);
} catch {
  console.error(`Cannot read manifest directory: ${dir}`);
  process.exit(1);
}

const jsonFiles = entries.filter((f) => f.endsWith(".json"));
if (jsonFiles.length === 0) {
  console.warn(`No manifest files found in ${dir}`);
  process.exit(0);
}

const merged = {};
for (const file of jsonFiles) {
  const raw = await readFile(join(dir, file), "utf8");
  let entry;
  try {
    entry = JSON.parse(raw);
  } catch {
    console.warn(`Skipping malformed manifest file: ${file}`);
    continue;
  }
  if (entry.file && Array.isArray(entry.testids)) {
    merged[entry.file] = entry.testids;
  }
}

const out = resolve(outputFile);
await writeFile(out, JSON.stringify(merged, null, 2) + "\n");
console.log(`Wrote ${Object.keys(merged).length} file(s) to ${out}`);
