//! Internal identifiers for recognized and unrecognized keystore entries.

use std::path::PathBuf;

use tor_basic_utils::PathExt;
#[cfg(feature = "key-encoding")]
use tor_key_forge::KeystoreItemType;

#[cfg(feature = "key-encoding")]
use crate::ArtiPath;

/// The raw identifier of a key inside a [`Keystore`](crate::Keystore).
///
/// The exact type of the identifier depends on the backing storage of the keystore
/// (for example, an on-disk keystore will identify its entries by [`Path`](RawEntryId::Path)).
#[cfg_attr(feature = "experimental-api", visibility::make(pub))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RawEntryId {
    /// An entry identified by path inside an on-disk keystore.
    // NOTE: this will only be used by on-disk keystores like
    // [`ArtiNativeKeystore`](crate::ArtiNativeKeystore)
    Path(PathBuf),

    /// An entry of an in-memory ephemeral key storage
    /// [`ArtiEphemeralKeystore`](crate::ArtiEphemeralKeystore)
    ///
    // TODO: the concept of a "raw identifier" doesn't really make sense
    // in the context of the `ArtiEphemeralKeystore`,
    // which is why this "raw" identifier is of exactly the same type
    // (`(ArtiPath, KeystoreItemType)`) as its non-"raw" counterpart.
    // Ephemeral keystores are just in-memory key-value mappings;
    // unlike file system-based keystores, these don't have entries with "raw"
    // identifiers that need to be validated and parsed before they can be used.
    //
    // We might want to remove this variant entirely,
    // and make `RawEntryId` optional in e.g. `KeystoreEntry`.

    #[cfg(feature = "key-encoding")]
    Ephemeral((ArtiPath, KeystoreItemType)),
    // TODO: when/if we add support for non on-disk keystores,
    // new variants will be added
}

impl core::fmt::Display for RawEntryId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Path(inner) => core::fmt::Display::fmt(&(inner.display_lossy()), formatter),
            #[cfg(feature = "key-encoding")]
            Self::Ephemeral(inner) => {
                formatter.write_fmt(format_args!("{0} {1:?}", inner.0, inner.1))
            }
        }
    }
}
