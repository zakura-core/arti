ADDED (relocated API): public module `oneshot`, containing the existing
`Canceled` reexport, `Sender<T>` and `Receiver<T>` aliases, and
`channel<T>() -> (Sender<T>, Receiver<T>)` constructor formerly supplied by
`oneshot-fused-workaround`. Direct users of that standalone package must use
`tor_basic_utils::oneshot` or the retained `tor_async_utils::oneshot` reexport.
The standalone package and its empty `full` feature are removed.

`Sender<T>` remains `futures::channel::oneshot::Sender<T>`. `Receiver<T>` remains
`futures::future::Fuse<futures::channel::oneshot::Receiver<T>>`; `Canceled` is
still the same futures type. All associated trait implementations and methods
therefore retain their underlying Rust types. `channel` still calls the same
futures constructor and fuses its receiver. No fields, variants, constants,
trait methods or `pub(crate)` items are added or widened. The added regression
test helpers are private.

The pinned futures 0.3.34 receiver still reports termination before its sender
cancellation has been observed. The ordinary tests retain an upstream control
and verify that the wrapper delivers cancellation once through `select!`,
handles cancellation after a pending poll and successful delivery, remains
terminated after consumption, and notifies the sender when its receiver drops.

The implementation and these tests move without functional changes. Existing
public aliases and signatures in other retained crates still use the same
`Sender`, `Fuse<Receiver>` and `Canceled` types; only their import paths change.
`tor_async_utils::oneshot` remains available with the same contents.

This crate gains the existing futures dependency with `std` only; test builds
also enable futures `async-await` for the selection control. No runtime provider
or executor feature is required here. `tor-rtmock` promotes its existing
`tor-basic-utils` dev edge to a normal edge and forwards its `full` selector.
Other consumers already depended on these basic utilities normally. There is
no dependency cycle or replacement compatibility crate.

ADDED: `is_valid_hostname(&str) -> bool` checks ASCII hostname syntax without
normalization or resolution. Client-address parsing and network-document
hostname parsing share this helper while retaining their existing numeric-name
policies. No existing signature, trait implementation or feature changes.

CHANGED: The `test_rng` module now requires the existing `full` feature outside
this crate's unit tests. This affects its `TestingRng` reexport, `testing_rng`
function, and `Config` enum, including the `Random`, `Deterministic`, and
`Seeded` variants, `from_env` and `into_rng` methods, and derived `Debug`,
`Clone`, `Copy`, `Eq`, and `PartialEq` implementations. Their signatures,
provider types, deterministic streams, and environment handling are unchanged.
Existing `full` builds retain all of this testing API.

The test RNG's `rand_chacha` dependency is now optional, enabled by `full` or
this crate's dev dependency. No new named or implicit Cargo features, public
or `pub(crate)` items, constructors, signatures, or trait implementations are
introduced. Production random number generation is unchanged.
