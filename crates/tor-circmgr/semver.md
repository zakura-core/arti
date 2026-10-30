
BREAKING: Remove geoip and its optional country argument on CircMgr::get_or_launch_exit. Remove crate-visible usage::CountryCode, TargetTunnelUsage::Exit.country_code, SupportedTunnelUsage::Exit.country_code, TorPath::country_code, ExitPathBuilder::in_given_country and the corresponding internal get_or_launch_exit argument. The former feature-off signatures and selection rules remain.

BREAKING: `Isolation` now extends `std::any::Any` instead of
`downcast_rs::Downcast`. Its generated `is`, `downcast`, `downcast_rc`,
`downcast_ref` and `downcast_mut` methods, and inherited `into_any`,
`into_any_rc`, `as_any` and `as_any_mut`, are removed. Upcast a trait object
to `dyn Any` for standard checked downcasts. Isolation compatibility and join
rules, including rejection of different concrete isolation types, are unchanged.
