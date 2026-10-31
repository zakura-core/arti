BREAKING: Deriving `TorConfig` now passes through any `#[non_exhaustive]` (bugfix)

BREAKING: Remove the `load-config` feature and configuration-file discovery,
loading and resolution APIs: `sources`, `ConfigurationSource`,
`ConfigurationSources`, `ConfigurationTree`, `load::resolve*`, `load::TopLevel`,
`ConfigError`, `ConfigGetValueError` and their members. Remove Figment re-exports.
Typed builder traits, defaults, validation and reconfiguration remain.

BREAKING: Remove command-line TOML overrides: the cmdline module, CmdLine type and ConfigurationSources::{from_cmdline, try_from_cmdline, push_option, options}. Structured builders remain.
