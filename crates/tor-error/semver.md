BREAKING: Remove the unused `rpc` Cargo feature marker. The existing
`experimental` and `experimental-api` selectors no longer forward that marker.
Error types, variants, trait methods, signatures and visibility are unchanged.

BREAKING: Consolidate `retry-error` into this error-handling facade. The
standalone package and its empty `full` feature are removed. Its implementation
and existing tests move into a private module; no replacement package is added.

Public API path migration:
- `retry_error::RetryError<E>` becomes `tor_error::RetryError<E>`.
- `retry_error::fmt_error_with_sources` becomes
  `tor_error::fmt_error_with_sources`.

`RetryError` retains its private fields and all existing bounds and signatures:
`in_attempt_to`, `push_timed`, `push`, `sources`, `len`, `is_empty`, `extend`,
`dedup_by`, `extend_from_retry_error` and `dedup`. Its `Clone`, `Debug`,
`std::error::Error`, `Display` and `IntoIterator` implementations are unchanged;
`IntoIterator::{Item = E, IntoIter = std::vec::IntoIter<E>, into_iter}` stay the
same. The formatter retains `(&dyn Error, &mut fmt::Formatter) -> fmt::Result`.
No fields, methods, variants, constants or `pub(crate)` items are added or
widened. The implementation module remains private.

The existing public payload types in `tor_circmgr::Error::RequestFailed` and
`tor_hsclient::ConnError::{DescriptorDownload, Failed}` consequently refer to the
`tor_error`-owned retry type. Their variant names, generic payloads and behavior
are unchanged. Downstream code naming the former standalone type must update
its import path to the `tor_error`-owned type.

`tor-error` takes over the existing `humantime` dependency and the same
wasm32-unknown `web-time` dependency. Timestamp sources, formatting, error-chain
deduplication, overflow handling, retry aggregation and iteration are unchanged.
Clippy's bans on implicit-clock `push`/`extend` follow the new public path.
