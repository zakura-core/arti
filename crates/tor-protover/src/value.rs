//! `Value`, the numerical part of a protocol capability

use super::*;

/// Maximum allowable value for a protocol's version field.
//
// TODO this should be a u8, not a usize, probably
pub(crate) const MAX_VER: usize = 63;

/// "Value", the numerical part of a protocol capability
//
// Invariant: valid, ie in the range 1..=MAX_VER.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)] //

pub struct Value(u8);

impl core::convert::From<Value> for u8 {
    #[inline]
    fn from(value: Value) -> Self {
        <u8 as core::convert::From<_>>::from(value.0)
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _0 = &self.0;
        core::fmt::Display::fmt(_0, formatter)
    }
}

/// A `Value` was out of range (zero, or >63)
#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("Protocol version {v} out of range")]
pub struct ValueOutOfRange {
    /// The value that was wrong
    v: u8,
}

impl core::convert::From<ValueOutOfRange> for u8 {
    #[inline]
    fn from(value: ValueOutOfRange) -> Self {
        <u8 as core::convert::From<_>>::from(value.v)
    }
}

impl Value {
    /// Create a `Value` from a raw integer
    pub const fn new(v: u8) -> Result<Self, ValueOutOfRange> {
        if 1 <= v && v <= MAX_VER as _ {
            Ok(Value(v))
        } else {
            Err(ValueOutOfRange { v })
        }
    }

    /// Create a `Value` from a raw integer, in const context
    ///
    /// # Panics
    ///
    /// Panics if `v` is out of range.
    ///
    /// Not ideal in non-const contexts if panicking is remotely possible,
    /// since the panic message is very bad.
    pub const fn new_const(v: u8) -> Self {
        match Self::new(v) {
            Ok(y) => y,
            Err(ValueOutOfRange { .. }) => panic!(),
        }
    }

    /// Return a mask with precisely this bit set
    pub fn to_bit(self) -> u64 {
        1_u64 << self.0
    }

    /// Iterate over all legal `Value`s
    pub fn iter_all() -> impl Iterator<Item = Self> + Clone {
        (1..=(MAX_VER as _)).map(Value::new_const)
    }
}

impl TryFrom<u8> for Value {
    type Error = ValueOutOfRange;
    fn try_from(v: u8) -> Result<Self, ValueOutOfRange> {
        Self::new(v)
    }
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::mixed_attributes_style)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_time_subtraction)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::needless_pass_by_value)]
    #![allow(clippy::string_slice)] // See arti#2571
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->

    use super::*;

    #[test]
    fn simple() {
        let good = |v| assert_eq!(Value::try_from(v), Ok(Value(v)));
        let bad = |v| assert_eq!(Value::try_from(v), Err(ValueOutOfRange { v }));
        bad(0);
        good(1);
        good(63);
        bad(64);
        bad(255);
    }
}
