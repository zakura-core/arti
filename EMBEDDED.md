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
