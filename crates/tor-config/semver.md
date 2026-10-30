BREAKING: Deriving `TorConfig` now passes through any `#[non_exhaustive]` (bugfix)

ADDED: Default-enabled `load-config` feature. Disabling it removes the
configuration-file loading methods, `ConfigurationTree`, resolution APIs,
command-line TOML parsing and Figment re-exports. Programmatic builders,
configuration source discovery and configuration errors remain available.

BREAKING: Remove command-line TOML overrides: the cmdline module, CmdLine type and ConfigurationSources::{from_cmdline, try_from_cmdline, push_option, options}. File sources and structured builders remain.
