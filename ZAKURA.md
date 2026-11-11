# Zakura's Arti fork

This repository maintains the embedded Arti Tor client used by Zakura's wallet
libraries. The upstream repository is
[Tor Project's Arti](https://gitlab.torproject.org/tpo/core/arti).
The starting upstream commit is `d46be843e`; upstream history and licenses are
preserved. Dependency reductions are reviewed as separate pull requests, with
compatibility tests for affected behavior.

Published fork packages have a `zakura-` prefix and an initial `-zakura.1`
prerelease version. Rust library names and dependency aliases stay
the same, so source code can retain imports such as `arti_client` and
`tor_rtcompat`. The fork and upstream packages are distinct Rust type identities;
applications that pass Arti or runtime types across crate boundaries must use one
family consistently. Guard-state provenance records the fork package and version.

`maint/zakura/package-family.json` records the upstream and fork package names and
versions. Run `python3 maint/zakura/check-package-family.py` after updating from
upstream. It rejects forgotten package aliases, mismatched registry fallbacks and
mixed upstream/fork lockfile identities, including target and development
dependencies. Internal path dependencies retain version requirements for the
same fork packages, so packaging cannot silently substitute upstream Arti.

The initial package namespace does not itself remove dependencies. Consumer graph
changes are measured separately.

For local validation use the pinned Rust toolchain:

```sh
python3 maint/zakura/check-package-family.py
cargo fmt --all -- --check
cargo test --locked -p zakura-tor-config --no-default-features
cargo test --locked -p zakura-tor-config
cargo check --locked -p zakura-arti-client --no-default-features \
  --features tokio,rustls,onion-service-client
```

The package namespace applies to the retained embedded workspace described in
[EMBEDDED.md](EMBEDDED.md). Standalone application packages are omitted.

## Publishing

The family is on crates.io, and new versions publish through the `Publish crates`
workflow with Trusted Publishing. Bump the versions in the manifests and
`package-family.json`, merge to `main`, then dispatch the workflow from `main`.
`verify` mode builds every unpublished version without credentials and checks the
crates.io token exchange. `publish` mode runs the same build, then uploads after
the `crates-io` environment is approved. Versions already on crates.io are
skipped, so dispatching again after a failure publishes only what is left.

crates.io limits how quickly one publisher can upload new versions of existing
crates, and a release that bumps the whole family goes past that burst.
`maint/zakura/publish-family.py` waits for the retry time crates.io reports and
continues.
