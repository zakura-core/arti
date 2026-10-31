
BREAKING: Remove geoip and its optional country argument on CircMgr::get_or_launch_exit. Remove crate-visible usage::CountryCode, TargetTunnelUsage::Exit.country_code, SupportedTunnelUsage::Exit.country_code, TorPath::country_code, ExitPathBuilder::in_given_country and the corresponding internal get_or_launch_exit argument. The former feature-off signatures and selection rules remain.

BREAKING: `Isolation` now extends `std::any::Any` instead of
`downcast_rs::Downcast`. Its generated `is`, `downcast`, `downcast_rc`,
`downcast_ref` and `downcast_mut` methods, and inherited `into_any`,
`into_any_rc`, `as_any` and `as_any_mut`, are removed. Upcast a trait object
to `dyn Any` for standard checked downcasts. Isolation compatibility and join
rules, including rejection of different concrete isolation types, are unchanged.

BREAKING: `StreamIsolationBuilder::build` returns `StreamIsolation`
directly, since both fields have defaults and construction cannot fail.
Remove the unused `StreamIsolationBuilderError` enum, its
`UninitializedField` and `ValidationError` variants/payloads, and its
`Debug`, `Display`, `Error`, `From<String>`,
`From<derive_builder::UninitializedFieldError>`, and
`From<derive_builder::SubfieldBuildError<E>>` implementations.
The explicit builder retains its setters, constructor, `Clone`, and
`Default` without adding any public or crate-visible name.
