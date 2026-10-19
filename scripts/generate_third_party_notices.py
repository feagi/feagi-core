#!/usr/bin/env python3
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
"""Generate the third-party notice file required to ship FEAGI.

When a component is offered under a choice of licenses and Apache-2.0 is one
of them, Neuraville elects Apache-2.0. Otherwise the generator elects the
most permissive option that is not GPL, LGPL, or AGPL. A component whose only
license is copyleft fails the run.

Usage:
  python3 scripts/generate_third_party_notices.py --manifest Cargo.toml --output THIRD_PARTY_NOTICES.txt
  python3 scripts/generate_third_party_notices.py --manifest src-tauri/Cargo.toml --output path --npm package-lock.json --check
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

# First match wins. Apache-2.0 is first so every dual-licensed crate that
# offers it is taken under Apache-2.0, including "MIT OR Apache-2.0 OR LGPL".
_PREFERENCE: tuple[str, ...] = (
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "MIT",
    "MIT-0",
    "BSD-3-Clause",
    "BSD-2-Clause",
    "ISC",
    "0BSD",
    "Zlib",
    "BSL-1.0",
    "Unlicense",
    "CC0-1.0",
    "NCSA",
    "CDLA-Permissive-2.0",
    "Unicode-3.0",
    "Python-2.0",
    "OFL-1.1",
    "CC-BY-4.0",
    "MPL-2.0",
)

_COPYLEFT_MARKERS: tuple[str, ...] = ("GPL", "LGPL", "AGPL")

# Used only when no upstream package ships a standalone file for this id.
# libfuzzer-sys declares NCSA alongside Apache-2.0 but does not vendor the text.
_FALLBACK_LICENSE_TEXT: dict[str, str] = {
    "NCSA": """University of Illinois/NCSA Open Source License

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal with
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

Redistributions of source code must retain the above copyright notice, this
list of conditions and the following disclaimers.

Redistributions in binary form must reproduce the above copyright notice, this
list of conditions and the following disclaimers in the documentation and/or
other materials provided with the distribution.

Neither the names of the copyright holders nor the names of their contributors
may be used to endorse or promote products derived from this Software without
specific prior written permission.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
CONTRIBUTORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS WITH THE
SOFTWARE.""",
}


def _split_top(expr: str, separator: str) -> list[str]:
    """Split on `separator` at parenthesis depth 0."""
    depth = 0
    parts: list[str] = []
    start = 0
    index = 0
    while index < len(expr):
        char = expr[index]
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif depth == 0 and expr.startswith(separator, index):
            parts.append(expr[start:index].strip())
            index += len(separator)
            start = index
            continue
        index += 1
    parts.append(expr[start:].strip())
    return [part for part in parts if part]


def _strip_wrapping_parens(expr: str) -> str:
    text = expr.strip()
    while text.startswith("(") and text.endswith(")"):
        depth = 0
        wraps = True
        for index, char in enumerate(text):
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            if depth == 0 and index != len(text) - 1:
                wraps = False
                break
        if not wraps or depth != 0:
            break
        text = text[1:-1].strip()
    return text


def _is_copyleft(license_id: str) -> bool:
    upper = license_id.upper()
    return any(marker in upper for marker in _COPYLEFT_MARKERS)


def elect_license(expression: str) -> str:
    """Return the license Neuraville distributes this component under.

    AND clauses are all kept. OR clauses resolve to Apache-2.0 when it is
    offered, otherwise to the first entry in `_PREFERENCE`.
    """
    text = _strip_wrapping_parens(expression.replace(" / ", " OR ").replace("/", " OR "))
    and_parts = _split_top(text, " AND ")
    if len(and_parts) > 1:
        return " AND ".join(elect_license(part) for part in and_parts)
    or_parts = _split_top(text, " OR ")
    if len(or_parts) > 1:
        options: list[str] = []
        for part in or_parts:
            try:
                options.append(elect_license(part))
            except ValueError:
                # A copyleft alternative is kept so a permissive sibling can win.
                options.append(_strip_wrapping_parens(part))
        for preferred in _PREFERENCE:
            if preferred in options:
                return preferred
        permissive = [option for option in options if not _is_copyleft(option)]
        if not permissive:
            raise ValueError(f"no permissive license in {expression!r}")
        return permissive[0]
    if _is_copyleft(text):
        raise ValueError(f"copyleft-only license {expression!r}")
    return text


def _registry_roots() -> list[Path]:
    cargo_home = Path.home() / ".cargo" / "registry" / "src"
    return sorted(cargo_home.glob("index.crates.io-*"))


def _crate_dir(name: str, version: str) -> Path | None:
    for root in _registry_roots():
        candidate = root / f"{name}-{version}"
        if candidate.is_dir():
            return candidate
    return None


def _license_files(directory: Path) -> list[Path]:
    found: list[Path] = []
    for path in sorted(directory.iterdir()):
        if not path.is_file():
            continue
        name = path.name.lower()
        if name.startswith(("license", "licence", "copying", "notice")) or name == "authors":
            found.append(path)
    return found


def _copyrights(text: str) -> list[str]:
    """Copyright lines only. Skips the word "copyright" inside license boilerplate."""
    lines: list[str] = []
    for raw in text.splitlines()[:80]:
        line = raw.strip().lstrip("#").strip()
        lower = line.lower()
        if not lower.startswith("copyright"):
            continue
        if "(c)" not in lower and not any(char.isdigit() for char in line):
            continue
        if any(phrase in lower for phrase in ("shall ", "above copyright", "copyright notice", "copyright holder")):
            continue
        if line not in lines:
            lines.append(line)
        if len(lines) == 4:
            break
    return lines


def _cargo_packages(manifest: Path) -> list[dict[str, object]]:
    raw = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--offline", "--manifest-path", str(manifest)],
        cwd=manifest.parent,
    )
    data = json.loads(raw)
    packages: list[dict[str, object]] = []
    for package in data["packages"]:
        if package.get("source") is None:
            continue
        packages.append(package)
    return packages


def _npm_packages(lock_path: Path) -> list[tuple[str, str, str, Path]]:
    """Production npm packages as (name, version, expression, directory)."""
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    rows: list[tuple[str, str, str, Path]] = []
    root = lock_path.parent
    for rel, meta in lock.get("packages", {}).items():
        if not rel.startswith("node_modules/") or meta.get("dev") is True:
            continue
        if rel.startswith("node_modules/@neuraville/"):
            continue
        directory = root / rel
        expression = meta.get("license")
        if isinstance(expression, dict):
            expression = expression.get("type")
        if not expression:
            license_file = directory / "LICENSE"
            if license_file.is_file() and "MIT" in license_file.read_text(encoding="utf-8", errors="replace")[:80]:
                expression = "MIT"
        if not expression or not isinstance(expression, str):
            raise ValueError(f"npm package {rel} has no license")
        name = rel.removeprefix("node_modules/")
        version = str(meta.get("version") or "")
        rows.append((name, version, expression, directory))
    return rows


def _component_block(
    kind: str,
    name: str,
    version: str,
    elected: str,
    copyrights: list[str],
    license_body: str,
    notice_body: str,
) -> tuple[str, str, str]:
    """Return (index line, license-text key, license body to keep once)."""
    rights = "; ".join(copyrights) if copyrights else "copyright stated in the upstream license file"
    source = ""
    if elected == "MPL-2.0" and kind == "crate":
        source = f" Source: https://crates.io/crates/{name}/{version} (unmodified)."
    index = f"- {name} {version} ({kind}): {elected}. {rights}.{source}"
    return index, elected, license_body if not notice_body else license_body + "\n\nNOTICE:\n" + notice_body


def _remember_license(texts: dict[str, str], elected: str, body: str) -> None:
    """Keep one license text per atomic id, taken from a package under only that id."""
    for atomic in elected.split(" AND "):
        texts.setdefault(atomic, "")
    if " AND " not in elected and body and not texts.get(elected):
        texts[elected] = body


def build_notices(manifest: Path, npm_lock: Path | None) -> str:
    """Build the full notice document."""
    index_lines: list[str] = []
    license_texts: dict[str, str] = {}
    failures: list[str] = []

    for package in sorted(_cargo_packages(manifest), key=lambda item: (str(item["name"]), str(item["version"]))):
        name = str(package["name"])
        version = str(package["version"])
        expression = str(package.get("license") or "")
        try:
            elected = elect_license(expression)
        except ValueError as error:
            failures.append(f"{name} {version}: {error}")
            continue
        directory = _crate_dir(name, version)
        copyrights: list[str] = []
        body = ""
        notice = ""
        if directory is not None:
            for path in _license_files(directory):
                text = path.read_text(encoding="utf-8", errors="replace")
                lowered = path.name.lower()
                if lowered.startswith("notice"):
                    notice = text.strip()
                elif lowered == "authors":
                    copyrights.extend(line for line in _copyrights(text) if line not in copyrights)
                else:
                    copyrights.extend(line for line in _copyrights(text) if line not in copyrights)
                    if not body:
                        body = text.strip()
        if not copyrights:
            authors = package.get("authors") or []
            if isinstance(authors, list) and authors:
                copyrights.append("Copyright " + ", ".join(str(author) for author in authors[:3]))
        index, _key, _kept = _component_block("crate", name, version, elected, copyrights, body, notice)
        index_lines.append(index)
        _remember_license(license_texts, elected, body)

    if npm_lock is not None:
        for name, version, expression, directory in sorted(_npm_packages(npm_lock)):
            try:
                elected = elect_license(expression)
            except ValueError as error:
                failures.append(f"{name} {version}: {error}")
                continue
            copyrights = []
            body = ""
            if directory.is_dir():
                for path in _license_files(directory):
                    text = path.read_text(encoding="utf-8", errors="replace")
                    copyrights.extend(line for line in _copyrights(text) if line not in copyrights)
                    if not body and not path.name.lower().startswith("notice"):
                        body = text.strip()
            index, _key, _kept = _component_block("npm", name, version, elected, copyrights, body, "")
            index_lines.append(index)
            _remember_license(license_texts, elected, body)

    if failures:
        raise SystemExit("license election failed:\n" + "\n".join(failures))

    lines = [
        "Third-party notices",
        "",
        "Neuraville elects Apache-2.0 for every component that offers Apache-2.0",
        "as one of its licenses. Where Apache-2.0 is not offered, the notice below",
        "names the permissive license that applies. GPL, LGPL, and AGPL are never",
        "elected. MPL-2.0 components are used unmodified; their source is the",
        "published crate named in the component line.",
        "",
        "Components",
        "",
        *index_lines,
        "",
        "License texts",
        "",
    ]
    for license_id in sorted(license_texts):
        lines.append(f"===== {license_id} =====")
        lines.append("")
        body = license_texts[license_id] or _FALLBACK_LICENSE_TEXT.get(license_id, "")
        if not body:
            raise SystemExit(f"no license text for {license_id}")
        lines.append(body)
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def _self_check() -> None:
    """Election rules that keep copyleft out of the distribution."""
    assert elect_license("MIT OR Apache-2.0 OR LGPL-2.1-or-later") == "Apache-2.0"
    assert elect_license("MIT OR Apache-2.0") == "Apache-2.0"
    assert elect_license("Apache-2.0/MIT") == "Apache-2.0"
    assert elect_license("MIT OR GPL-3.0-only") == "MIT"
    assert elect_license("(MIT OR Apache-2.0) AND Unicode-3.0") == "Apache-2.0 AND Unicode-3.0"
    assert elect_license("MPL-2.0") == "MPL-2.0"
    try:
        elect_license("GPL-3.0-only")
    except ValueError:
        return
    raise AssertionError("GPL-only license was accepted")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate third-party license notices")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--npm", type=Path, default=None)
    parser.add_argument("--check", action="store_true", help="Exit 1 when the output file is stale")
    parser.add_argument("--self-check", action="store_true", help="Run election assertions and exit")
    args = parser.parse_args()
    if args.self_check:
        _self_check()
        print("OK: license election")
        return
    if args.manifest is None or args.output is None:
        parser.error("--manifest and --output are required")
    document = build_notices(args.manifest.resolve(), args.npm.resolve() if args.npm else None)
    output = args.output.resolve()
    if args.check:
        current = output.read_text(encoding="utf-8") if output.is_file() else ""
        if current != document:
            print(f"ERROR: {output} is stale. Regenerate it.", file=sys.stderr)
            sys.exit(1)
        print(f"OK: {output}")
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(document, encoding="utf-8")
    print(f"Wrote {output}")


if __name__ == "__main__":
    main()
