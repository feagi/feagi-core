import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  sortVersionsNewestFirst,
  versionsFromSnapshotFilenames,
  writeCrateGraphIndex,
} from "../scripts/crate-graph-index.mjs";

test("snapshot filenames become versions and index.json is ignored", () => {
  assert.deepEqual(
    versionsFromSnapshotFilenames([
      "index.json",
      "v0.0.12.json",
      "notes.txt",
      "v0.0.9.json",
    ]),
    ["0.0.12", "0.0.9"]
  );
});

test("versions sort newest first", () => {
  assert.deepEqual(sortVersionsNewestFirst(["0.0.9", "0.0.12", "0.0.10"]), [
    "0.0.12",
    "0.0.10",
    "0.0.9",
  ]);
});

test("writeCrateGraphIndex records every snapshot in the directory", () => {
  const directory = mkdtempSync(join(tmpdir(), "crate-graphs-"));
  writeFileSync(join(directory, "v0.0.9.json"), "{}\n");
  writeFileSync(join(directory, "v0.0.12.json"), "{}\n");

  const versions = writeCrateGraphIndex(directory);
  const index = JSON.parse(readFileSync(join(directory, "index.json"), "utf8"));

  assert.deepEqual(versions, ["0.0.12", "0.0.9"]);
  assert.deepEqual(index, { versions: ["0.0.12", "0.0.9"] });
});
