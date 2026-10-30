# Embedded wallet source scope

This fork retains `arti-client`, `fs-mistrust`, `tor-rtcompat` and their
complete local dependency closure, including optional, target-specific, build
and development dependencies. Embedded Tor and ordinary onion connections stay
available. Shared libraries needed by optional client features remain present.

Standalone Arti applications, relay/directory applications, RPC applications,
the web documentation build, Python bindings, Debian packaging and standalone
examples are omitted. Upstream Git history and licenses remain available.

Run `python3 maint/zakura/check-embedded-source.py` to verify retained path
dependencies, including fuzz and development manifests. These source deletions
reduce this fork's workspace and maintenance scope; they do not remove packages
from a downstream wallet's already selected dependency graph.

Serde's string/display and map adapters select `serde_with`'s `alloc` and
`macros` features explicitly. They do not need its `std` feature, which weakly
resolves unused timezone and schema adapter dependencies. Serialization formats
and the adapters themselves are unchanged.

## Remaining maintenance commands

The retained executables maintain library inputs; wallet clients do not depend
on them. `maint/keygen-openssh-test` regenerates the malformed and valid key
fixtures in `tor-keymgr/testdata`. `maint/fixup-features` checks Cargo feature
relationships during fork updates. The optional `tor-geoip` export tool converts
GeoIP data for the embedded database. None is an application-facing wallet CLI.

The removed client-auth fixture generator only targeted tests of the deleted
Arti application. Command-line configuration overrides and raw key-management
CLI adapters are omitted; structured configuration and typed key APIs remain.
