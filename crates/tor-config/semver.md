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

BREAKING: Remove `Flatten<T, U>` (including its tuple constructor, fields and
trait implementations), `Flattenable::has_field`, the exported `Flattenable`
derive-deftly template and doc-hidden `flattenable_extract_fields`. `TorConfig`
no longer derives `Flattenable` for builders or accepts `no_flattenable_trait`.
The memory-quota `ConfigBuilder` no longer implements `Flattenable` either.

BREAKING: Remove the doc-hidden `deps::serde_value` and
`derive::exports::serde_value` re-exports. Conditionally unavailable fields
now reject explicitly supplied values during deserialization, including null,
rather than buffering and later warning or rejecting during `build()`. These
fields are omitted from serialized builders. Supported fields, typed builder
setters, validation and defaults are unchanged. The generated private marker
for an unavailable field is now `Option<()>`, without retaining input data.

As before, field-level Serde customization is not forwarded to unavailable
placeholders; container-level field naming still applies. No public helper,
feature, constructor or trait method is added.

BREAKING: Remove the doc-hidden `deps::educe` re-export. The exported
`define_map_builder!` macro now implements `Deref` and `DerefMut` directly,
with the same target and behavior. Existing configuration traits, defaults,
Serde support and feature availability are unchanged.

BREAKING: Remove the unused listening-server configuration helper `Listen`
and its root re-export. This removes its `new_none`, `new_localhost`,
`new_localhost_optional`, `is_empty`, `using_auto`, `using_port_zero`,
`ip_addrs`, `localhost_port_legacy`, `single_address_legacy`,
`is_loopback_only` and `is_localhost_only` methods. Its `Clone`, `Hash`,
`Debug`, `Ord`, `PartialOrd`, `Eq`, `PartialEq`, `Serialize`, `Deserialize`,
`Default` and `Display` implementations disappear, along with conversions
to/from its private representation. The opaque tuple field was private;
the three named constructors and `Default` were its public constructors.

Remove the root re-export and public non-exhaustive empty struct
`ListenUnsupported`, including its `Debug`, `Clone`, `Display` and `Error`
implementations. It had no public fields or constructor. No public constants,
aliases, variants or crate-private declarations change, and no API is added.
Typed outbound-client builders, storage, permissions, memory quotas, padding,
protocol fields and persisted-state formats are unchanged. Remove the now-unused
direct `either` dependency; other retained packages still depend on it. No Cargo
feature is removed.
