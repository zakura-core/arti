#!/usr/bin/env python3
"""Reject mixed upstream/fork package identities before publishing or building."""

import json
import os
from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parents[2]
FAMILY = json.loads((Path(__file__).parent / "package-family.json").read_text())


def dependencies(manifest):
    for kind in ("dependencies", "build-dependencies", "dev-dependencies"):
        yield from manifest.get(kind, {}).items()
    for target in manifest.get("target", {}).values():
        yield from dependencies(target)


def main():
    errors = []
    found = set()
    for directory, children, files in os.walk(ROOT):
        children[:] = [name for name in children if name not in {"target", ".git"}]
        if "Cargo.toml" not in files:
            continue
        path = Path(directory) / "Cargo.toml"
        manifest = tomllib.loads(path.read_text())
        package = manifest.get("package", {})
        relative = str(path.parent.relative_to(ROOT))
        if package and package.get("publish") is not False:
            found.add(relative)
            expected = FAMILY.get(relative)
            if expected is None:
                errors.append(f"{relative}: publishable package is not in the fork family")
                continue
            for field in ("name", "version"):
                if package[field] != expected[field]:
                    errors.append(f"{relative}: unexpected package {field}")
            if (path.parent / "src/lib.rs").exists():
                library = manifest.get("lib", {}).get("name")
                if library != expected["upstream_name"].replace("-", "_"):
                    errors.append(f"{relative}: Rust library name changed")
        for alias, dependency in dependencies(manifest):
            if not isinstance(dependency, dict) or "path" not in dependency:
                continue
            target = (path.parent / dependency["path"]).resolve()
            try:
                target_relative = str(target.relative_to(ROOT))
            except ValueError:
                errors.append(f"{relative}: {alias} has a path outside this repository")
                continue
            expected = FAMILY.get(target_relative)
            if expected is None:
                continue
            if dependency.get("package", alias) != expected["name"]:
                errors.append(f"{relative}: {alias} points at an upstream package name")
            # Fuzz packages and the existing dev-only self-dependency can omit
            # their registry fallback; Cargo strips the latter when packaging.
            if (
                package.get("publish") is not False
                and target != path.parent.resolve()
                and "version" not in dependency
            ):
                errors.append(f"{relative}: {alias} is missing its registry version")
            if "version" in dependency and dependency["version"] != expected["version"]:
                errors.append(f"{relative}: {alias} has a different registry version")
    if found != FAMILY.keys():
        errors.append(f"stale family entries: {sorted(FAMILY.keys() - found)}")
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    upstream = {entry["upstream_name"] for entry in FAMILY.values()}
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    for name, profile in manifest.get("profile", {}).items():
        for package in profile.get("package", {}):
            if package in upstream:
                errors.append(f"profile {name} retains upstream package: {package}")
    for package in lock["package"]:
        if package["name"] in upstream:
            errors.append(f"lockfile retains upstream identity: {package['name']}")
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"package family: {len(FAMILY)} distinct Zakura packages; Rust names preserved")


if __name__ == "__main__":
    main()
