#!/usr/bin/env python3
"""Check a fresh embedded client without workspace test-feature unification."""

import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[2]
FAMILY = json.loads((Path(__file__).parent / "package-family.json").read_text())


def main():
    with tempfile.TemporaryDirectory(prefix="zakura-arti-consumer-") as directory:
        consumer = Path(directory)
        dependencies = []
        for name, defaults, features in (
            ("arti-client", False, ["tokio", "rustls", "onion-service-client"]),
            ("fs-mistrust", True, []),
            ("tor-rtcompat", True, ["tokio", "rustls"]),
        ):
            path = "crates/" + name
            package = FAMILY[path]["name"]
            target = json.dumps(str(ROOT / path), ensure_ascii=False)
            dependencies.append(
                f'{name} = {{ package = "{package}", path = {target}, '
                f'default-features = {str(defaults).lower()}, features = {json.dumps(features)} }}'
            )
        manifest = consumer / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "fresh-zakura-tor-consumer"\nversion = "0.0.0"\n'
            'edition = "2024"\n[workspace]\n[dependencies]\n'
            + "\n".join(dependencies) + "\n"
        )
        (consumer / "src").mkdir()
        (consumer / "src/main.rs").write_text(
            "use arti_client::{TorClient, config::TorClientConfig};\n"
            "use tor_rtcompat::tokio::TokioRustlsRuntime;\n"
            "fn accepts_runtime(runtime: TokioRustlsRuntime) {\n"
            "    let _builder = TorClient::with_runtime(runtime);\n"
            "}\n"
            "fn main() {\n"
            "    let mut builder = TorClientConfig::builder();\n"
            "    let permissions = |_: &mut fs_mistrust::MistrustBuilder| {};\n"
            "    permissions(builder.storage().permissions());\n"
            "    let _config = builder.build().expect(\"default configuration\");\n"
            "    let _runtime_type: fn(TokioRustlsRuntime) = accepts_runtime;\n"
            "}\n"
        )
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
        tree = subprocess.check_output(
            ["cargo", "tree", "--manifest-path", str(manifest), "--edges", "normal,build",
             "--prefix", "none", "--format", "{p}"],
            cwd=ROOT, env=environment, text=True,
        )
        packages = {line.removesuffix(" (*)") for line in tree.splitlines()}
        names = {package.split()[0] for package in packages}
        forbidden = {
            "url", "idna", "idna_adapter", "sysinfo", "notify", "notify-types",
            "criterion", "criterion-cycles-per-byte", "tor-rtmock", "zakura-tor-rtmock",
            "liblzma", "liblzma-sys", "zstd", "zstd-safe", "zstd-sys", "jobserver",
        }
        forbidden.update(entry["upstream_name"] for entry in FAMILY.values())
        unexpected = (names & forbidden) | {name for name in names if name.startswith("icu_")}
        required = {FAMILY["crates/" + name]["name"] for name in ("arti-client", "fs-mistrust", "tor-rtcompat")}
        if unexpected or not required <= names:
            raise SystemExit(f"unexpected dependencies: {sorted(unexpected)}; missing: {sorted(required - names)}")
        subprocess.run(
            ["cargo", "check", "--locked", "--manifest-path", str(manifest)],
            cwd=ROOT, env=environment, check=True,
        )
        print(f"embedded client: {len(packages) - 1} dependency packages; graph and public type boundaries passed")


if __name__ == "__main__":
    main()
