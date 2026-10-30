BREAKING: Deriving `TorConfig` now passes through any `#[non_exhaustive]` (bugfix)

ADDED: Default-enabled `load-config` feature. Disabling it removes the
configuration-file loading methods, `ConfigurationTree`, resolution APIs,
command-line TOML parsing and Figment re-exports. Programmatic builders,
configuration source discovery and configuration errors remain available.
