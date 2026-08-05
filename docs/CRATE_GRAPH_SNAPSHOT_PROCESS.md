# Crate Graph Snapshot Process

## Overview

This document describes the tooling that generates versioned dependency graph snapshots for the
feagi-core Rust workspace and how those snapshots are published to the BrainsForRobots portal
(`nrs-portal`) at `/feagi/architecture`.

The process has two parts:

1. **Extraction** — a Node.js script (`scripts/generate-crate-graph.mjs`) reads the workspace
   via `cargo metadata` and writes a versioned JSON file.
2. **Publication** — the JSON file is committed to `nrs-portal/src/data/crate-graphs/` and
   the website page serves it alongside a version list sourced from the GitHub releases API.

---

## Files Involved

### feagi-core (this repo)

| Path | Purpose |
|------|---------|
| `scripts/generate-crate-graph.mjs` | Extraction script — runs `cargo metadata` and writes the JSON snapshot |
| `scripts/crate-categories.json` | Maps each crate name to its architectural layer category |
| `docs/crate-graphs/v{version}.json` | Generated snapshots; committed to version control |

### nrs-portal

| Path | Purpose |
|------|---------|
| `src/data/crate-graphs/v{version}.json` | Snapshots served at build time (copied from feagi-core) |
| `src/app/feagi/architecture/page.tsx` | Next.js server component; fetches GitHub releases and loads snapshots |
| `src/components/feagi/architecture/CrateGraph.tsx` | Interactive SVG dependency graph (client component) |
| `src/components/feagi/architecture/ArchitectureView.tsx` | Version picker shell (client component) |

---

## Generating a Snapshot

Run from the `feagi-core` directory:

```bash
node scripts/generate-crate-graph.mjs
```

The script:

1. Executes `cargo metadata --format-version 1 --no-deps` inside the workspace.
2. Filters packages to workspace members only.
3. Reads `scripts/crate-categories.json` to assign each crate to an architectural layer.
4. Separates required dependencies from optional / feature-gated ones using the `optional` flag
   from `Cargo.toml`.
5. Writes the output to `docs/crate-graphs/v{workspace_version}.json`.

To write to a custom path:

```bash
node scripts/generate-crate-graph.mjs --out path/to/output.json
```

### Output schema

```json
{
  "version": "0.0.12",
  "generated_at": "2026-06-18T01:23:45.000Z",
  "crates": [
    {
      "id": "feagi-config",
      "label": "feagi-config",
      "category": "foundation",
      "description": "Configuration loader for FEAGI — cross-platform TOML-based configuration"
    }
  ],
  "required_edges": [
    { "from": "feagi-structures", "to": "feagi-serialization" }
  ],
  "optional_edges": [
    { "from": "feagi-npu-burst-engine", "to": "feagi-services" }
  ]
}
```

---

## Publishing a New Version

After cutting a release in the `feagi/feagi` GitHub repository:

1. Generate the snapshot:
   ```bash
   cd feagi-core
   node scripts/generate-crate-graph.mjs
   ```

2. Copy the output to the website:
   ```bash
   cp docs/crate-graphs/v{version}.json \
      ../nrs-portal/src/data/crate-graphs/v{version}.json
   ```

3. Commit and deploy the `nrs-portal`. The `/feagi/architecture` page will:
   - Pull the release list from `https://api.github.com/repos/feagi/feagi/releases`.
   - Show any release in the version picker for which a local snapshot file exists.
   - Fall back to local snapshots only if the GitHub API is unavailable.

No changes to website code are required unless the crate structure itself changes significantly.

---

## Maintaining `crate-categories.json`

`scripts/crate-categories.json` is the only manually maintained configuration file. It maps
each crate name to one of these category keys:

| Key | Displayed label | Color |
|-----|----------------|-------|
| `foundation` | Foundation | teal |
| `npu` | NPU Core | purple |
| `hal` | HAL | orange |
| `algorithms` | Algorithms | green |
| `io` | I/O & Agents | yellow |
| `services` | Services / API | rose |
| `training` | Training | blue |
| `umbrella` | Umbrella | bright blue |

When a new crate is added to the workspace, add its name as a key in `crate-categories.json`
with the appropriate category value. If no entry exists, the crate falls back to `"other"` and
renders with a neutral grey color.

---

## How the Website Page Works

`/feagi/architecture` is a Next.js server component with `revalidate = 3600` (ISR, re-fetched
hourly). On each server render:

1. The GitHub releases API is queried for all published (non-draft, non-prerelease) tags in
   `feagi/feagi`.
2. All local snapshot files in `src/data/crate-graphs/` are loaded.
3. The version picker shows the intersection: GitHub-confirmed releases that have a local
   snapshot. Any local snapshot not yet on GitHub (e.g. a pre-release build) appears at the
   bottom of the list.
4. The selected snapshot is passed as a prop to the `CrateGraph` client component, which
   computes a `dagre` layout and renders an interactive SVG.

---

## Adding a New Architectural Category

1. Add the category key to `scripts/crate-categories.json` for all relevant crates.
2. Add an entry to `CATEGORY_META` in
   `nrs-portal/src/components/feagi/architecture/CrateGraph.tsx`:
   ```ts
   newcategory: { label: "Display Name", color: "#hexcolor" },
   ```
3. Regenerate the snapshot and publish following the steps above.
