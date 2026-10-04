# Crate Graph Snapshot Process

## Overview

This document describes the tooling that generates versioned dependency graph snapshots for the
feagi-core Rust workspace and how those snapshots are published to the BrainsForRobots portal
(`nrs-portal`) at `/feagi/architecture`.

The process has two parts:

1. **Extraction** — a Node.js script (`scripts/generate-crate-graph.mjs`) reads the workspace
   via `cargo metadata` and writes a versioned JSON file plus `docs/crate-graphs/index.json`.
2. **Publication** — the feagi-core release workflow commits those files on the public
   `staging` branch and tags that commit. `nrs-portal` reads them from the public
   repository. It does not store a copy.

---

## Files Involved

### feagi-core (this repo)

| Path | Purpose |
|------|---------|
| `scripts/generate-crate-graph.mjs` | Extraction script — runs `cargo metadata` and writes the JSON snapshot |
| `scripts/crate-categories.json` | Maps each crate name to its architectural layer category |
| `docs/crate-graphs/v{version}.json` | Generated snapshots; committed to version control |
| `docs/crate-graphs/index.json` | Newest-first version list. The portal fetches this file, then each snapshot |

### nrs-portal

| Path | Purpose |
|------|---------|
| `src/lib/feagi/crateGraphSnapshots.ts` | Fetches `index.json` and the listed snapshots from the public feagi-core repo |
| `src/app/feagi/architecture/page.tsx` | Next.js server component; renders the fetched snapshots or the fetch error |
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
5. Writes `docs/crate-graphs/v{workspace_version}.json` when that version is not already recorded.
6. Rewrites `docs/crate-graphs/index.json` from the snapshot filenames in that directory.

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

`.github/workflows/release_to_crates_io.yml` does this on the publish branch (`staging`)
before the release tag is created:

1. Run `node scripts/generate-crate-graph.mjs`.
2. If `docs/crate-graphs/v{version}.json` is new, or `index.json` changed, commit those files
   and push them to `staging`.
3. Tag that commit and publish the crates from it.

An existing `v{version}.json` is left in place, so a rerun of the same version does not
rewrite the snapshot.

The portal reads:

`https://raw.githubusercontent.com/feagi/feagi-core/staging/docs/crate-graphs/index.json`

and then each `v{version}.json` listed there. The responses are cached for one hour.
A failed request is shown on the page. There is no copy of the JSON in `nrs-portal`.

To regenerate locally:

```bash
cd feagi-core
node scripts/generate-crate-graph.mjs
```

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

`/feagi/architecture` loads snapshots from the public feagi-core repository at request time.
Cached responses are reused for one hour.

1. Fetch `docs/crate-graphs/index.json` from the `staging` branch.
2. Fetch each listed `v{version}.json` in that order.
3. Pass the snapshots to `CrateGraph`, which computes a `dagre` layout and renders an SVG.
4. If either request fails, the page shows that error and does not substitute another graph.

---

## Adding a New Architectural Category

1. Add the category key to `scripts/crate-categories.json` for all relevant crates.
2. Add an entry to `CATEGORY_META` in
   `nrs-portal/src/components/feagi/architecture/CrateGraph.tsx`:
   ```ts
   newcategory: { label: "Display Name", color: "#hexcolor" },
   ```
3. Regenerate the snapshot. The next feagi-core release commits it.
