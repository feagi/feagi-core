/**
 * Builds docs/crate-graphs/index.json from the versioned snapshot files
 * already in that directory. The portal reads this index; it does not scan
 * the directory itself.
 */

import { readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const SNAPSHOT_FILENAME = /^v(.+)\.json$/;

/**
 * @param {string[]} filenames
 * @returns {string[]}
 */
export function versionsFromSnapshotFilenames(filenames) {
  const versions = [];
  for (const name of filenames) {
    if (name === "index.json") {
      continue;
    }
    const match = SNAPSHOT_FILENAME.exec(name);
    if (match) {
      versions.push(match[1]);
    }
  }
  return versions;
}

/**
 * Newest version first. Numeric compare so 0.0.12 sorts ahead of 0.0.9.
 *
 * @param {string[]} versions
 * @returns {string[]}
 */
export function sortVersionsNewestFirst(versions) {
  return [...versions].sort((a, b) =>
    b.localeCompare(a, undefined, { numeric: true })
  );
}

/**
 * Rewrites index.json beside the snapshot files.
 *
 * @param {string} directory
 * @returns {string[]}
 */
export function writeCrateGraphIndex(directory) {
  const versions = sortVersionsNewestFirst(
    versionsFromSnapshotFilenames(readdirSync(directory))
  );
  const indexPath = join(directory, "index.json");
  writeFileSync(
    indexPath,
    JSON.stringify({ versions }, null, 2) + "\n"
  );
  return versions;
}
