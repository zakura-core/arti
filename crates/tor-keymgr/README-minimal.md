# tor-keymgr without key-store encodings

This configuration provides key-path specifiers and configuration builders
without SSH key parsing. The key-manager and key-store interfaces use their
existing dummy implementations: reads return no key, and writes fail.

Enable `keymgr` (enabled by default) for functional key stores. It includes the
`key-encoding` feature, which provides key conversion, SSH-related re-exports
and their error variants. `key-encoding` can also be selected independently.

In-memory onion authentication keys remain available from `tor-hscrypto`
without `key-encoding`. File-based key-store operations require `keymgr`.
