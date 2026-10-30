#!/usr/bin/env python3
"""Reject missing path dependencies in retained manifests, including fuzz targets."""
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[2]
for manifest in ROOT.rglob("Cargo.toml"):
    if "target" in manifest.relative_to(ROOT).parts:
        continue
    data = tomllib.loads(manifest.read_text())
    tables = [data, *data.get("target", {}).values()]
    for table in tables:
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            for alias, declaration in table.get(kind, {}).items():
                if isinstance(declaration, dict) and "path" in declaration:
                    target = manifest.parent / declaration["path"] / "Cargo.toml"
                    if not target.is_file():
                        raise SystemExit(f"{manifest.relative_to(ROOT)}: {alias} has no manifest")
print("all retained path dependencies exist")
