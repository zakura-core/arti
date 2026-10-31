//! Misc helper functions and types for use in parsing network documents

use derive_deftly::define_derive_deftly;

mod system_time;
pub(crate) use system_time::{saturating_add, saturating_sub};

pub(crate) mod rangemap_ext;
pub(crate) mod str;

pub mod batching_split_before;

use std::iter::Peekable;

define_derive_deftly! {
    /// Implement `AsMut<Self>`
    ///
    /// For Reasons, Rust does not have a blanket:
    ///
    /// ```rust,ignore
    /// impl<T> AsMut<T> for T { .. }
    /// ```
    ///
    /// This derive macro expands to the obvious and trivial implementation,
    /// for the type that it's applied to.
    //
    // TODO move this somewhere lower in the stack, eg tor-basic-utils
    export AsMutSelf expect items:

    impl<$tgens> ::std::convert::AsMut<Self> for $ttype where $twheres {
        fn as_mut(&mut self) -> &mut Self {
            self
        }
    }
}

/// An iterator with a `.peek()` method
///
/// We make this a trait to avoid entangling all the types with `Peekable`.
/// Ideally we would do this with `Itertools::PeekingNext`
/// but that was not implemented for `&mut PeekingNext`
/// when we wrote this code,
/// and we need that because we use a lot of `&mut NetdocReader`.
/// <https://github.com/rust-itertools/itertools/issues/678>
///
/// TODO: As of itertools 0.11.0, `PeekingNext` _is_ implemented for
/// `&'a mut I where I: PeekingNext`, so we can remove this type some time.
///
/// # **UNSTABLE**
///
/// This type is UNSTABLE and not part of the semver guarantees.
/// You'll only see it if you ran rustdoc with `--document-private-items`.
// This is needed because this is a trait bound for batching_split_before.
#[doc(hidden)]
pub trait PeekableIterator: Iterator {
    /// Inspect the next item, if there is one
    fn peek(&mut self) -> Option<&Self::Item>;
}

impl<I: Iterator> PeekableIterator for Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        self.peek()
    }
}

impl<I: PeekableIterator> PeekableIterator for &mut I {
    fn peek(&mut self) -> Option<&Self::Item> {
        <I as PeekableIterator>::peek(*self)
    }
}

/// A Private module for declaring a "sealed" trait.
pub(crate) mod private {
    /// A non-exported trait, used to prevent others from implementing a trait.
    ///
    /// For more information on this pattern, see [the Rust API
    /// guidelines](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed).
    #[expect(dead_code, unreachable_pub)] // TODO keep this Sealed trait in case we want it again?
    pub trait Sealed {}
}

/// Implement string serialization and parsing, including UTF-8 byte input.
macro_rules! impl_serde_via_string {
    ($type:ty) => {
        impl serde::Serialize for $type {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                serializer.collect_str::<&Self>(&self)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D: serde::Deserializer<'de>>(
                deserializer: D,
            ) -> std::result::Result<Self, D::Error> {
                /// Parse the serialized string representation.
                struct StringVisitor;

                impl serde::de::Visitor<'_> for StringVisitor {
                    type Value = $type;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        std::fmt::Display::fmt("a string", formatter)
                    }

                    fn visit_str<E: serde::de::Error>(
                        self,
                        value: &str,
                    ) -> std::result::Result<Self::Value, E> {
                        value.parse().map_err(E::custom)
                    }

                    fn visit_bytes<E: serde::de::Error>(
                        self,
                        value: &[u8],
                    ) -> std::result::Result<Self::Value, E> {
                        let value = std::str::from_utf8(value).map_err(E::custom)?;
                        self.visit_str(value)
                    }
                }

                deserializer.deserialize_str(StringVisitor)
            }
        }
    };
}

#[cfg(test)]
#[allow(unused)]
fn test_as_mut_compiles() {
    use derive_deftly::Deftly;

    #[derive(Deftly)]
    #[derive_deftly(AsMutSelf)]
    struct S<T: Clone>
    where
        Option<T>: Clone,
    {
        t: T,
    }

    let _: &mut S<()> = S { t: () }.as_mut();
}
