//! Traits used by typed configuration builders.

use crate::ConfigBuildError;

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
