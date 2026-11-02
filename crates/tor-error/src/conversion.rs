//! Errors for extracting enum variants and parsing their names.

use std::{error::Error, fmt};

/// An enum value did not contain the variant required by a conversion.
#[derive(Clone, Copy, Debug)]
pub struct TryIntoError<T> {
    /// The owned value that could not be converted.
    pub input: T,
    /// Names of the variants accepted by the conversion.
    variant_names: &'static str,
    /// Name of the conversion's output type.
    output_type: &'static str,
}

impl<T> TryIntoError<T> {
    /// Describe an unsuccessful conversion while retaining its owned input.
    #[must_use]
    #[inline]
    pub const fn new(input: T, variant_names: &'static str, output_type: &'static str) -> Self {
        Self {
            input,
            variant_names,
            output_type,
        }
    }
}

impl<T> fmt::Display for TryIntoError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Only {} can be converted to {}",
            self.variant_names, self.output_type,
        )
    }
}

impl<T: fmt::Debug> Error for TryIntoError<T> {}

/// A string did not name a variant accepted by an enum's parser.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FromStrError {
    /// Name of the enum being parsed.
    type_name: &'static str,
}

impl FromStrError {
    /// Describe an unrecognized string representation of `type_name`.
    #[must_use]
    #[inline]
    pub const fn new(type_name: &'static str) -> Self {
        Self { type_name }
    }
}

impl fmt::Display for FromStrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Invalid `{}` string representation",
            self.type_name,
        )
    }
}

impl Error for FromStrError {}
