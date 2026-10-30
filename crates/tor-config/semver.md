BREAKING: Deriving `TorConfig` now passes through any `#[non_exhaustive]` (bugfix)

ADDED: Default-enabled `load-config` feature. Disabling it removes the
configuration-file loader, `ConfigurationTree`, resolution APIs and Figment
re-exports, while preserving programmatic builders and configuration errors.
