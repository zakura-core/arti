BREAKING: Deriving `TorConfig` now passes through any `#[non_exhaustive]` (bugfix)

BREAKING: Remove the `load-config` feature and configuration-file discovery,
loading and resolution APIs: `sources`, `ConfigurationSource`,
`ConfigurationSources`, `ConfigurationTree`, `load::resolve*`, `load::TopLevel`,
`ConfigError`, `ConfigGetValueError` and their members. Remove Figment re-exports.
Typed builder traits, defaults, validation and reconfiguration remain.

BREAKING: Remove command-line TOML overrides: the cmdline module, CmdLine type and ConfigurationSources::{from_cmdline, try_from_cmdline, push_option, options}. Structured builders remain.


BREAKING: Remove `From<derive_builder::UninitializedFieldError>` and
`From<derive_builder::SubfieldBuildError<ConfigBuildError>>` for
`ConfigBuildError`, including their `from` methods. The legacy builder fork
is no longer a normal or development dependency. Construct the existing
`ConfigBuildError` variants directly, or use the existing `TorConfig` builder
support and `ConfigBuildError::within` for nested field prefixes.

The configuration errors, list/map helpers, standard-builder macro, exported
`ExtendBuilder` template and current `TorConfig` template retain their public
and crate-visible declarations and behavior. No public or crate-visible API,
feature or helper is added. Existing helper/default/Serde/error-prefix tests
and examples now use `TorConfig` or explicit Rust builders.
