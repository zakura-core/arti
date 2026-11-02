# Wallet client scope

This fork applies received consensus diffs. The consensus-diff generation API
and its LCS provider have been removed.

Removed public API:

- `gen_cons_diff(&str, &str, DiffSizeStrictness) -> Result<String, Error>`.
- `Error::InvalidInput(tor_netdoc::parse2::ParseError)` and
  `Error::Bug(tor_error::Bug)`, which were produced only during generation.
- The corresponding `From<tor_netdoc::parse2::ParseError>` and
  `From<tor_error::Bug>` implementations for `Error`.

Removed crate-visible API:

- `GenEdDiffError`, including `MissingUnixLineEnding { lno: usize }`,
  `ContainsDotLine { lno: usize }`, and `Write(std::fmt::Error)`, and its derived
  `From<std::fmt::Error>` implementation.

`apply_diff`, `looks_like_diff`, `DiffResult`, `DiffSizeStrictness` (including
`Generate`), and the `slow-diff-apply` API retain their behavior and signatures.
The `full` feature now forwards only to `tor-llcrypto/full`; the direct
`tor-error` and `tor-netdoc` dependencies belonged to generation.

The existing random round-trip and consensus-vector tests use independent
whole-document replacement diffs. The former generation-input tests exercise
the applier's dot terminators, missing terminators, and CRLF normalization.
The parser, application, digest-verification and fuzz comparison implementations
are unchanged.
