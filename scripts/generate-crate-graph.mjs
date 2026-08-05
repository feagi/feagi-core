#!/usr/bin/env node
/**
 * generate-crate-graph.mjs
 *
 * Generates a versioned JSON snapshot of the feagi-core workspace dependency
 * graph by running `cargo metadata` and enriching the output with category
 * data from crate-categories.json.
 *
 * Usage:
 *   node scripts/generate-crate-graph.mjs
 *   node scripts/generate-crate-graph.mjs --out docs/crate-graphs/v0.0.13.json
 *
 * Output schema:
 *   {
 *     "version": "0.0.12",
 *     "generated_at": "2026-06-18T...",
 *     "crates": [ { id, label, category, description } ],
 *     "required_edges": [ { from, to } ],
 *     "optional_edges": [ { from, to } ]
 *   }
 */

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const WORKSPACE_ROOT = resolve(__dirname, "..");
const CATEGORIES_FILE = join(__dirname, "crate-categories.json");

// Parse --out flag
const outFlagIndex = process.argv.indexOf("--out");
let outPath = null;
if (outFlagIndex !== -1 && process.argv[outFlagIndex + 1]) {
  outPath = resolve(process.argv[outFlagIndex + 1]);
}

// Run cargo metadata
console.log("Running cargo metadata...");
const raw = execSync("cargo metadata --format-version 1 --no-deps", {
  cwd: WORKSPACE_ROOT,
  maxBuffer: 10 * 1024 * 1024,
}).toString();

const metadata = JSON.parse(raw);
const categories = JSON.parse(readFileSync(CATEGORIES_FILE, "utf8"));

// Identify workspace members by their package ID
const workspaceMemberIds = new Set(metadata.workspace_members);
const workspacePackages = metadata.packages.filter((p) =>
  workspaceMemberIds.has(p.id)
);

// Build a name → package map for quick lookup
const byName = new Map(workspacePackages.map((p) => [p.name, p]));
const workspaceNames = new Set(byName.keys());

// Derive workspace version from the root "feagi" package or any package
const rootPkg = byName.get("feagi") ?? workspacePackages[0];
const workspaceVersion = rootPkg?.version ?? "unknown";

// Build crate nodes
const crates = workspacePackages
  .sort((a, b) => a.name.localeCompare(b.name))
  .map((pkg) => ({
    id: pkg.name,
    label: pkg.name === "feagi" ? "feagi  (umbrella)" : pkg.name,
    category: categories[pkg.name] ?? "other",
    description: pkg.description ?? "",
  }));

// Build edges — only between workspace members
const requiredEdges = [];
const optionalEdges = [];

for (const pkg of workspacePackages) {
  for (const dep of pkg.dependencies) {
    // Skip non-workspace deps, dev deps, and build deps
    if (!workspaceNames.has(dep.name)) continue;
    if (dep.kind === "dev" || dep.kind === "build") continue;

    const edge = { from: dep.name, to: pkg.name };

    if (dep.optional) {
      optionalEdges.push(edge);
    } else {
      requiredEdges.push(edge);
    }
  }
}

const snapshot = {
  version: workspaceVersion,
  generated_at: new Date().toISOString(),
  crates,
  required_edges: requiredEdges,
  optional_edges: optionalEdges,
};

// Determine output path
const defaultOut = join(
  WORKSPACE_ROOT,
  "docs",
  "crate-graphs",
  `v${workspaceVersion}.json`
);
const targetPath = outPath ?? defaultOut;

mkdirSync(dirname(targetPath), { recursive: true });
writeFileSync(targetPath, JSON.stringify(snapshot, null, 2) + "\n");

console.log(`Wrote ${crates.length} crates, ${requiredEdges.length} required edges, ${optionalEdges.length} optional edges`);
console.log(`Output: ${targetPath}`);
