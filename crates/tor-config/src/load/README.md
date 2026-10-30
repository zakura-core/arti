Processing a `ConfigurationTree` into a validated configuration

This module, and particularly [`resolve`], takes care of:

  * Deserializing a [`ConfigurationTree`](crate::ConfigurationTree) into various `FooConfigBuilder`
  * Calling the `build()` methods to get various `FooConfig`.
  * Reporting unrecognised configuration keys
    (eg to help the user detect misspellings).

This is step 3 of the overall config processing,
as described in the [crate-level documentation](crate).

# Starting points

To use this, you will need to:

  * `#[derive(Builder)]` and use [`impl_standard_builder!`](crate::impl_standard_builder)
    for all of your configuration structures,
    using `#[sub_builder]` etc. sa appropriate,
    and making your builders [`Deserialize`](serde::Deserialize).

  * [`impl TopLevel`](TopLevel) for your *top level* structures (only).

  * Call [`resolve`] (or one of its variants) with a `ConfigurationTree`,
    to obtain your top-level configuration(s).

# Example

In this example the developers are embedding `arti`, `arti_client`, etc.,
into a program of their own.  The example code shown:

 * Defines a configuration structure `EmbedderConfig`,
   for additional configuration settings for the added features.
 * Establishes some configuration sources
   (the trivial empty [`ConfigurationSources`](crate::ConfigurationSources),
   to avoid clutter in the example)
 * Reads those sources into a single configuration taxonomy [`ConfigurationTree`](crate::ConfigurationTree).
 * Processes that configuration into a 3-tuple of configuration
   structs for the three components, namely:
     - `TorClientConfig`, the configuration for the `arti_client` crate's `TorClient`
     - `ArtiConfig`, for behaviours in the `arti` command line utility
     - `EmbedderConfig`.
 * Will report a warning to the user about config settings found in the config files,
   but not recognized by *any* of the three config consumers,

```
# fn main() -> Result<(), tor_config::load::ConfigResolveError> {
use derive_builder::Builder;
use tor_config::{impl_standard_builder, resolve, ConfigBuildError, ConfigurationSources};
use tor_config::load::TopLevel;
use tor_config::derive::prelude::*;
use serde::{Deserialize, Serialize};
use derive_deftly::Deftly;

#[derive(Debug, Clone, Deftly, Eq, PartialEq)]
#[derive_deftly(TorConfig)]
struct EmbedderConfig {
    // ....
}
impl TopLevel for EmbedderConfig {
    type Builder = EmbedderConfigBuilder;
}
#
# #[derive(Debug, Clone, Builder, Eq, PartialEq)]
# #[builder(build_fn(error = "ConfigBuildError"))]
# #[builder(derive(Debug, Serialize, Deserialize))]
# struct TorClientConfig { }
# impl_standard_builder! { TorClientConfig }
# impl TopLevel for TorClientConfig { type Builder = TorClientConfigBuilder; }
# impl tor_config::load::ConfigBuilder for TorClientConfigBuilder {
#     fn apply_defaults(&mut self) -> Result<(), ConfigBuildError> { Ok(()) }
# }
#
# #[derive(Debug, Clone, Builder, Eq, PartialEq)]
# #[builder(build_fn(error = "ConfigBuildError"))]
# #[builder(derive(Debug, Serialize, Deserialize))]
# struct ArtiConfig { }
# impl_standard_builder! { ArtiConfig }
# impl TopLevel for ArtiConfig { type Builder = ArtiConfigBuilder; }
# impl tor_config::load::ConfigBuilder for ArtiConfigBuilder {
#     fn apply_defaults(&mut self) -> Result<(), ConfigBuildError> { Ok(()) }
# }


let cfg_sources = ConfigurationSources::new_empty(); // In real program, use from_cmdline
let cfg = cfg_sources.load()?;

let (tcc, arti_config, embedder_config) =
     tor_config::resolve::<(TorClientConfig, ArtiConfig, EmbedderConfig)>(&cfg)?;

let _: EmbedderConfig = embedder_config; // etc.

# Ok(())
# }
```

