BREAKING: Abolished `DirectorySignaturesHashesAccu` field `sha1_unnamed`.
BREAKING: `Preamble`: `voting_delay` is now a mandatory field in the `Constructor`
BREAKING: `Preamble`: `voting_delay` is now a custom type `VotingDelay`
ADDED: `DocRelayFlag`, `DocRelayFlags::iter_incl_unknown`, `contains_impl_unknown`
ADDED: `DocRelayFlags` impl `FromIterator<DocRelayFlag>`
ADDED: `SupersededAuthorityKey::from_dir_source_and_key`
ADDED: `netstatus::ConsensusAuthoritySectionConstructor`, `DirSourceConstructor`
ADDED: `IdentifiedDigest` is now exhaustive, with fields `pub`
ADDED: `{FixedB64, FixedB16U}::from_byte_slice`
ADDED: `ProtoStatusesConstructor`; `ProtoStatuses` fields are now `pub`
ADDED: `ProtoStatusConstructor`; `ProtoStatus` fields are now `pub`

CHANGED: Private relay-nickname storage; public APIs, accepted inputs and serialized output are unchanged.

BREAKING: `types::relay_flags::RelayFlags` is a private-field domain set rather
than an alias for `enumset::EnumSet<RelayFlag>`. `DocRelayFlags::known`, its
`Deref` target, `ReprMode` constants, router-status `flags()` accessors and
builder `Into<RelayFlags>` arguments consequently use the new type. Code using
`EnumSet<RelayFlag>` must migrate. The former EnumSet `u16` layout/FFI contract
does not apply to the new Rust type. `RelayFlagsBits = u16` is unchanged.

The supported set interface is now `empty`, `all`, `contains`, `insert`, `iter`,
and the new `const from_flags(&[RelayFlag])` constructor. `contains` is now
`const`. `iter` returns an owning opaque `Iterator<Item = RelayFlag>`, rather
than `EnumSetIter`; its `Clone`, `Debug`, `DoubleEndedIterator` and
`ExactSizeIterator` interfaces are no longer exposed. Iteration order is
unchanged. Set `Copy`, `Clone`, `Default`, `Eq`, `PartialEq<Self>`, `Ord`,
`PartialOrd<Self>`, `Hash`, `Debug`, `From<RelayFlag>`, generic `|`/`|=`,
generic `&`/`&=`, and `!` remain. `Debug` now prints a standard debug set,
and set `Display` is removed.

The following inherited EnumSet methods are removed:

- `new`, `only`, `bit_width`, `variant_count`, `bit_index`, `is_bit_valid`,
  `len`, `is_empty`, `clear`, `remove`, `insert_all`, `remove_all`,
  `is_disjoint`, `is_superset`, `is_subset`, `union`, `intersection`,
  `difference`, `symmetric_difference`, `symmetrical_difference`, `complement`;
- `as_repr`, `from_repr`, `try_from_repr`, `from_repr_truncated`,
  `from_repr_unchecked`;
- `as_{u8,u16,u32,u64,u128,usize}`, `try_as_*`, `as_*_truncated`,
  `from_{u8,u16,u32,u64,u128,usize}`, `try_from_*`, `from_*_truncated`,
  `from_*_unchecked`;
- `as_array`, `try_as_array`, `as_array_truncated`, `from_array`,
  `try_from_array`, `from_array_truncated`, `from_array_unchecked`, `to_vec`,
  `copy_into_slice`, `try_copy_into_slice`, `copy_into_slice_truncated`,
  `from_slice`, `try_from_slice`, `from_slice_truncated`,
  `from_slice_unchecked`.

The set also loses `From<[RelayFlag; N]>`, owned `IntoIterator`, and `Extend`,
`FromIterator` and `Sum` for owned/borrowed flags and sets; cross-type
`PartialEq<RelayFlag>`/`PartialEq<MixedEnumSet<RelayFlag>>`; and `-`, `-=`, `^`,
`^=`. EnumSet's externally enabled `serde`/`defmt` implementations are no
longer available through feature unification.

`RelayFlag` keeps its variants, discriminants and ordinary derives, but no
longer implements `EnumSetType`, `EnumSetTypeWithRepr`, or EnumSet's private
helper traits. The derive-generated, doc-hidden `__EnumSetInitHelper` and
`__EnumSetOpHelper` types and their methods are removed. Flag `|` remains,
returning the domain set; flag `&`, `-`, `^`, `!`, and
`PartialEq<EnumSet<RelayFlag>>` are removed. `enum_set!` and
`MixedEnumSet<RelayFlag>` can no longer be used with this enum. Use
`RelayFlags::from_flags` for constants and `RelayFlag::A | RelayFlag::B` or
`RelayFlags::from(flag)` at runtime. No `pub(crate)` interfaces are changed.

BREAKING: Remove `hs-service`, `HsDescBuilder`,
`create_desc_sign_key_cert`, `IntroPointDesc::builder`,
`IntroPointDescBuilder`, and both generated builder error types and their
members/trait implementations. Onion descriptor publication and service
signing-key certificate generation are outside this wallet-client fork.
Client descriptor parsing/decryption, intro-point getters, and restricted
client authentication remain available.

REMOVED (internal): Descriptor publication layer structs/fields and their
`NetdocBuilder` implementations, plus service-only test fixture helpers.
`HsDescEncryption::encrypt` is now private test-only code; production
ciphertext processing retains the same decrypt/KDF/MAC implementation.
No public or crate-visible Rust API is added.
