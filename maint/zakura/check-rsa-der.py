#!/usr/bin/env python3
"""Keep the previous RSA DER serializer as a verification-only dependency."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def main():
    subject = ROOT / "crates/tor-llcrypto"
    package = tomllib.loads((subject / "Cargo.toml").read_text())["package"]["name"]
    with tempfile.TemporaryDirectory(prefix="arti-rsa-der-oracle-") as directory:
        consumer = Path(directory)
        manifest = consumer / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "rsa-der-counterpart"\nversion = "0.0.0"\n'
            'edition = "2024"\n[workspace]\n[dependencies]\n'
            f'subject = {{ package = {json.dumps(package)}, path = {json.dumps(str(subject))} }}\n'
            'rsa = "=0.9.10"\nsha1 = "=0.10.7"\n'
            'der-parser = { version = "=10.0.0", features = ["serialize"] }\n'
            '\n[[test]]\nname = "rsa_der"\n'
            f'path = {json.dumps(str(ROOT / "maint/zakura/probes/rsa-der.rs"))}\n'
        )
        environment = os.environ.copy()
        environment["RSA_DER_TESTDATA"] = str(subject / "testdata")
        subprocess.run(
            ["cargo", "test", "--manifest-path", str(manifest), "--", "--nocapture"],
            cwd=ROOT, env=environment, check=True,
        )
        locked = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
        if any(package["name"] == "cookie-factory" for package in locked):
            raise SystemExit("the verification-only serializer returned to the workspace lockfile")


if __name__ == "__main__":
    main()
