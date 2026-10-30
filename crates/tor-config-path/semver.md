REMOVED: The implicit `shellexpand` dependency feature. Use the existing
`expand-paths` feature for configuration path interpolation. Variable, default,
dollar-escape and tilde expansion behavior is unchanged; inserted paths retain
their native OS representation. No Rust signature or visibility changes.
