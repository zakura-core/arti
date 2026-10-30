
BREAKING: Remove geoip and its optional country argument on CircMgr::get_or_launch_exit. Remove crate-visible usage::CountryCode, TargetTunnelUsage::Exit.country_code, SupportedTunnelUsage::Exit.country_code, TorPath::country_code, ExitPathBuilder::in_given_country and the corresponding internal get_or_launch_exit argument. The former feature-off signatures and selection rules remain.
