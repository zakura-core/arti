#!/usr/bin/env python3
"""Keep the former nickname storage provider outside ordinary workspace builds."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def main():
    subject = ROOT / "crates/tor-netdoc"
    package = tomllib.loads((subject / "Cargo.toml").read_text())["package"]["name"]
    with tempfile.TemporaryDirectory(prefix="arti-nickname-compat-") as directory:
        consumer = Path(directory)
        manifest = consumer / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "nickname-compat"\nversion = "0.0.0"\n'
            'edition = "2024"\n[workspace]\n[dependencies]\n'
            f'tor-netdoc = {{ package = {json.dumps(package)}, '
            f'path = {json.dumps(str(subject))} }}\n'
            'tinystr = "=0.8.5"\nserde = "1"\nserde_json = "1"\n'
            '\n[[test]]\nname = "previous-provider"\n'
            f'path = {json.dumps(str(Path(__file__).with_name("nickname-compat.rs")))}\n'
        )
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
        subprocess.run(
            ["cargo", "test", "--manifest-path", str(manifest)],
            cwd=ROOT, env=environment, check=True,
        )


if __name__ == "__main__":
    main()
