//! Configuration builder traits and optional configuration-tree resolution.
//!
//! Programmatic builders work without the `load-config` feature. Enable it
//! to read configuration files and resolve them into validated configurations.
#![cfg_attr(feature = "load-config", doc = include_str!("load/README.md"))]

use crate::ConfigBuildError;
use serde::de::DeserializeOwned;

#[cfg(feature = "load-config")]
mod resolve;
#[cfg(feature = "load-config")]
#[cfg_attr(docsrs, doc(cfg(feature = "load-config")))]
pub use resolve::*;

/// A type that can be built from a builder.
pub trait Buildable {
    /// The type that constructs this Buildable.
    ///
    /// Typically, this type will implement [`Builder`].
    /// If it does, then `<Self::Builder>::Built` should be `Self`.
    type Builder;

    /// Return a new Builder for this type.
    fn builder() -> Self::Builder;
}

/// A type that can build some buildable type via a build method.
pub trait Builder {
    /// The type that this builder constructs.
    ///
    /// Typically, this type will implement [`Buildable`].
    /// If it does, then `<Self::Built as Buildable>::Builder` should be `Self`.
    type Built;

    /// Build into a `Built`
    ///
    /// Often shadows an inherent `build` method
    fn build(&self) -> Result<Self::Built, ConfigBuildError>;
}

/// A [`Builder`] as generated and used by our configuration system.
//
// (This is a separate type from Builder since we also implement Builder for some
// non-configuration builders.)
pub trait ConfigBuilder: Builder {
    /// Modify `self` by replacing any options that haven't been set with their defaults.
    ///
    /// Also, resolves deprecated settings:
    /// When a deprecated setting is provided, and the modern version is not,
    /// `apply_defaults` derives the modern value from the deprecated value,
    /// and writes it into the modern field.
    /// (If both a deprecated setting, and its modern form, are set
    /// on entry to `apply_defaults`, the deprecated form is ignored.)
    ///
    /// It is not necessary to call this method
    /// if all you want to do with this builder
    /// is to call [`build()`](Builder::build) on it:
    /// `build()` also takes defaults into account.
    fn apply_defaults(&mut self) -> Result<(), ConfigBuildError>;
}

/// Top-level configuration struct, made from a deserializable builder
///
/// One configuration consumer's configuration settings.
///
/// Implementing this trait only for top-level configurations,
/// which are to be parsed at the root level of a (TOML) config file taxonomy.
///
/// This trait exists to:
///
///  * Mark top-level configuration structures for use with `resolve`
///    (when `load-config` is enabled)
///  * Provide the type of the `Builder` for use by Rust generic code
pub trait TopLevel {
    /// The `Builder` which can be used to make a `Self`
    ///
    /// Should satisfy `&'_ Self::Builder: Builder<Built=Self>`
    type Builder: DeserializeOwned;

    /// Deprecated config keys, as "."-separates strings
    const DEPRECATED_KEYS: &'static [&'static str] = &[];
}
