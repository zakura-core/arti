//! Error type from parsing a document, and the position where it occurred
use thiserror::Error;

use crate::types::policy::PolicyError;
use std::{borrow::Cow, fmt, sync::Arc};

/// A position within a directory object. Used to tell where an error
/// occurred.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Pos {
    /// The error did not occur at any particular position.
    ///
    /// This can happen when the error is something like a missing entry:
    /// the entry is supposed to go _somewhere_, but we can't say where.
    None,
    /// The error occurred at an unknown position.
    ///
    /// We should avoid using this case.
    Unknown,
    /// The error occurred at an invalid offset within the string, or
    /// outside the string entirely.
    ///
    /// This can only occur because of an internal error of some kind.
    Invalid(usize),
    /// The error occurred at a particular byte within the string.
    ///
    /// We try to convert these to a Pos before displaying them to the user.
    Byte {
        /// Byte offset within a string.
        off: usize,
    },
    /// The error occurred at a particular line (and possibly at a
    /// particular byte within the line.)
    PosInLine {
        /// Line offset within a string.
        line: usize,
        /// Byte offset within the line.
        byte: usize,
    },
    /// The error occurred at a position in memory.  This shouldn't be
    /// exposed to the user, but rather should be mapped to a position
    /// in the string.
    Raw {
        /// A raw pointer to the position where the error occurred.
        ptr: *const u8,
    },
}

// It's okay to send a Pos to another thread, even though its Raw
// variant contains a pointer. That's because we never dereference the
// pointer: we only compare it to another pointer representing a
// string.
//
// TODO: Find a better way to have Pos work.
unsafe impl Send for Pos {}
unsafe impl Sync for Pos {}

impl Pos {
    /// Construct a Pos from an offset within a &str slice.
    #[allow(clippy::string_slice)] // TODO
    pub fn from_offset(s: &str, off: usize) -> Self {
        if off > s.len() || !s.is_char_boundary(off) {
            Pos::Invalid(off)
        } else {
            let s = &s[..off];
            let last_nl = s.rfind('\n');
            match last_nl {
                Some(pos) => {
                    let newlines = s.bytes().filter(|b| *b == b'\n').count();
                    Pos::PosInLine {
                        line: newlines + 1,
                        byte: off - pos,
                    }
                }
                None => Pos::PosInLine {
                    line: 1,
                    byte: off + 1,
                },
            }
        }
    }
    /// Construct a Pos from a slice of some other string.  This
    /// Pos won't be terribly helpful, but it may be converted
    /// into a useful Pos with `within`.
    pub fn at(s: &str) -> Self {
        let ptr = s.as_ptr();
        Pos::Raw { ptr }
    }
    /// Construct Pos from the end of some other string.
    #[allow(clippy::string_slice)] // TODO
    pub fn at_end_of(s: &str) -> Self {
        let ending = &s[s.len()..];
        Pos::at(ending)
    }
    /// Construct a position from a byte offset.
    pub fn from_byte(off: usize) -> Self {
        Pos::Byte { off }
    }
    /// Construct a position from a line and a byte offset within that line.
    pub fn from_line(line: usize, byte: usize) -> Self {
        Pos::PosInLine { line, byte }
    }
    /// If this position appears within `s`, and has not yet been mapped to
    /// a line-and-byte position, return its offset.
    pub(crate) fn offset_within(&self, s: &str) -> Option<usize> {
        match self {
            Pos::Byte { off } => Some(*off),
            Pos::Raw { ptr } => offset_in(*ptr, s),
            _ => None,
        }
    }
    /// Given a position, if it was at a byte offset, convert it to a
    /// line-and-byte position within `s`.
    ///
    /// Requires that this position was actually generated from `s`.
    /// If it was not, the results here may be nonsensical.
    ///
    /// TODO: I wish I knew an efficient safe way to do this that
    /// guaranteed that we always talking about the right string.
    #[must_use]
    pub fn within(self, s: &str) -> Self {
        match self {
            Pos::Byte { off } => Self::from_offset(s, off),
            Pos::Raw { ptr } => {
                if let Some(off) = offset_in(ptr, s) {
                    Self::from_offset(s, off)
                } else {
                    self
                }
            }
            _ => self,
        }
    }
}

/// If `ptr` is within `s`, return its byte offset.
fn offset_in(ptr: *const u8, s: &str) -> Option<usize> {
    // We need to confirm that 'ptr' falls within 's' in order
    // to subtract it meaningfully and find its offset.
    // Otherwise, we'll get a bogus result.
    //
    // Fortunately, we _only_ get a bogus result: we don't
    // hit unsafe behavior.
    let ptr_u = ptr as usize;
    let start_u = s.as_ptr() as usize;
    let end_u = (s.as_ptr() as usize) + s.len();
    if start_u <= ptr_u && ptr_u < end_u {
        Some(ptr_u - start_u)
    } else {
        None
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Pos::*;
        match self {
            None => write!(f, ""),
            Unknown => write!(f, " at unknown position"),
            Invalid(off) => write!(f, " at invalid offset at index {}", off),
            Byte { off } => write!(f, " at byte {}", off),
            PosInLine { line, byte } => write!(f, " on line {}, byte {}", line, byte),
            Raw { ptr } => write!(f, " at {:?}", ptr),
        }
    }
}

/// A variety of parsing error.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NetdocErrorKind {
    /// An internal error in the parser: these should never happen.
    Internal,
    /// Invoked an API in an incorrect manner.
    BadApiUsage,
    /// An entry was found with no keyword.
    MissingKeyword,
    /// An entry was found with no newline at the end.
    TruncatedLine,
    /// A bad string was found in the keyword position.
    BadKeyword,
    /// We found an ill-formed "BEGIN FOO" tag.
    BadObjectBeginTag,
    /// We found an ill-formed "END FOO" tag.
    BadObjectEndTag,
    /// We found a "BEGIN FOO" tag with an "END FOO" tag that didn't match.
    BadObjectMismatchedTag,
    /// We found a base64 object with an invalid base64 encoding.
    BadObjectBase64,
    /// The document is not supposed to contain more than one of some
    /// kind of entry, but we found one anyway.
    DuplicateToken,
    /// The document is not supposed to contain any of some particular kind
    /// of entry, but we found one anyway.
    UnexpectedToken,
    /// The document is supposed to contain any of some particular kind
    /// of entry, but we didn't find one one anyway.
    MissingToken,
    /// The document was supposed to have one of these, but not where we
    /// found it.
    MisplacedToken,
    /// We found more arguments on an entry than it is allowed to have.
    TooManyArguments,
    /// We didn't fine enough arguments for some entry.
    TooFewArguments,
    /// We found an object attached to an entry that isn't supposed to
    /// have one.
    UnexpectedObject,
    /// An entry was supposed to have an object, but it didn't.
    MissingObject,
    /// We found an object on an entry, but the type was wrong.
    WrongObject,
    /// We tried to find an argument that we were sure would be there,
    /// but it wasn't!
    ///
    /// This error should never occur in correct code; it should be
    /// caught earlier by TooFewArguments.
    MissingArgument,
    /// We found an argument that couldn't be parsed.
    BadArgument,
    /// We found an object that couldn't be parsed after it was decoded.
    BadObjectVal,
    /// There was some signature that we couldn't validate.
    BadSignature, // TODO(nickm): say which kind of signature.
    /// The object is not valid at the required time.
    BadTimeBound,
    /// There was a tor version we couldn't parse.
    BadTorVersion,
    /// There was an ipv4 or ipv6 policy entry that we couldn't parse.
    BadPolicy,
    /// An underlying byte sequence couldn't be decoded.
    Undecodable,
    /// Versioned document with an unrecognized version.
    BadDocumentVersion,
    /// Unexpected document type
    BadDocumentType,
    /// We expected a kind of entry that we didn't find
    MissingEntry,
    /// Document or section started with wrong token
    WrongStartingToken,
    /// Document or section ended with wrong token
    WrongEndingToken,
    /// Items not sorted as expected
    WrongSortOrder,
    /// A consensus lifetime was ill-formed.
    InvalidLifetime,
    /// Found an empty line in the middle of a document
    EmptyLine,
    /// The document began with a deprecated unicode BOM marker.
    BomMarkerFound,
    /// The document contained an internal NUL byte
    NulFound,
    /// An item contained extra spaces at a place where they are not allowed.
    ExtraneousSpace,
}

impl core::fmt::Display for NetdocErrorKind {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Internal => formatter.write_fmt(format_args!("internal error")),
            Self::BadApiUsage => formatter.write_fmt(format_args!("bad API usage")),
            Self::MissingKeyword => formatter.write_fmt(format_args!("no keyword for entry")),
            Self::TruncatedLine => {
                formatter.write_fmt(format_args!("line truncated before newline"))
            }
            Self::BadKeyword => formatter.write_fmt(format_args!("invalid keyword")),
            Self::BadObjectBeginTag => formatter.write_fmt(format_args!("invalid PEM BEGIN tag")),
            Self::BadObjectEndTag => formatter.write_fmt(format_args!("invalid PEM END tag")),
            Self::BadObjectMismatchedTag => {
                formatter.write_fmt(format_args!("mismatched PEM tags"))
            }
            Self::BadObjectBase64 => formatter.write_fmt(format_args!("invalid base64 in object")),
            Self::DuplicateToken => formatter.write_fmt(format_args!("duplicate entry")),
            Self::UnexpectedToken => formatter.write_fmt(format_args!("unexpected entry")),
            Self::MissingToken => formatter.write_fmt(format_args!("didn\'t find required entry")),
            Self::MisplacedToken => formatter.write_fmt(format_args!("entry out of place")),
            Self::TooManyArguments => formatter.write_fmt(format_args!("too many arguments")),
            Self::TooFewArguments => formatter.write_fmt(format_args!("too few arguments")),
            Self::UnexpectedObject => formatter.write_fmt(format_args!("unexpected object")),
            Self::MissingObject => formatter.write_fmt(format_args!("missing object")),
            Self::WrongObject => formatter.write_fmt(format_args!("wrong object type")),
            Self::MissingArgument => formatter.write_fmt(format_args!("missing argument")),
            Self::BadArgument => formatter.write_fmt(format_args!("bad argument for entry")),
            Self::BadObjectVal => formatter.write_fmt(format_args!("bad object for entry")),
            Self::BadSignature => formatter.write_fmt(format_args!("couldn\'t validate signature")),
            Self::BadTimeBound => {
                formatter.write_fmt(format_args!("couldn\'t validate time bound"))
            }
            Self::BadTorVersion => formatter.write_fmt(format_args!("couldn\'t parse Tor version")),
            Self::BadPolicy => formatter.write_fmt(format_args!("invalid policy entry")),
            Self::Undecodable => formatter.write_fmt(format_args!("decoding error")),
            Self::BadDocumentVersion => {
                formatter.write_fmt(format_args!("unrecognized document version"))
            }
            Self::BadDocumentType => formatter.write_fmt(format_args!("unexpected document type")),
            Self::MissingEntry => formatter.write_fmt(format_args!("missing entry")),
            Self::WrongStartingToken => formatter.write_fmt(format_args!("Wrong starting token")),
            Self::WrongEndingToken => formatter.write_fmt(format_args!("Wrong ending token")),
            Self::WrongSortOrder => formatter.write_fmt(format_args!("Incorrect sort order")),
            Self::InvalidLifetime => {
                formatter.write_fmt(format_args!("Invalid consensus lifetime"))
            }
            Self::EmptyLine => formatter.write_fmt(format_args!("Empty line")),
            Self::BomMarkerFound => {
                formatter.write_fmt(format_args!("unexpected byte-order marker"))
            }
            Self::NulFound => formatter.write_fmt(format_args!("unexpected NUL")),
            Self::ExtraneousSpace => formatter.write_fmt(format_args!("Extraneous spaces")),
        }
    }
}

/// The underlying source for an [`Error`](struct@Error).
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub(crate) enum NetdocErrorSource {
    /// An error when parsing a binary object.
    #[error("Error parsing binary object")]
    Bytes(#[from] tor_bytes::Error),
    /// An error when parsing an exit policy.
    #[error("Error parsing policy")]
    Policy(#[from] PolicyError),
    /// An error when parsing an integer.
    #[error("Couldn't parse integer")]
    Int(#[from] std::num::ParseIntError),
    /// An error when parsing an IP or socket address.
    #[error("Couldn't parse address")]
    Address(#[from] std::net::AddrParseError),
    /// An error when validating a signature.
    #[error("Invalid signature")]
    Signature(#[source] Arc<signature::Error>),
    /// An error when validating a signature on an embedded binary certificate.
    #[error("Invalid certificate")]
    CertSignature(#[from] tor_cert::CertError),
    /// An error caused by an expired or not-yet-valid descriptor.
    #[error("Descriptor expired or not yet valid")]
    UntimelyDescriptor(#[from] tor_checkable::TimeValidityError),
    /// Invalid protocol versions.
    #[error("Protocol versions")]
    Protovers(#[from] tor_protover::ParseError),
    /// A bug in our programming, or somebody else's.
    #[error("Internal error or bug")]
    Bug(#[from] tor_error::Bug),
}

/// Error parsing what was supposed to be a fixed, constant, string
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("expected fixed string {expected:?}, got {got:?}")]
// This is unlikely to change - and our macro expansion wants to be able to construct i.
// Making it exhaustive seems better than the bureaucracy of a constructor.
#[allow(clippy::exhaustive_structs)]
pub struct ExpectedConstantString {
    /// The fixed, constant string we expected
    pub expected: &'static str,

    /// The actual value we encountered
    pub got: String,
}

impl NetdocErrorKind {
    /// Construct a new Error with this kind.
    #[must_use]
    pub(crate) fn err(self) -> Error {
        Error {
            kind: self,
            msg: None,
            pos: Pos::Unknown,
            source: None,
        }
    }

    /// Construct a new error with this kind at a given position.
    #[must_use]
    pub(crate) fn at_pos(self, pos: Pos) -> Error {
        self.err().at_pos(pos)
    }

    /// Construct a new error with this kind and a given message.
    #[must_use]
    pub(crate) fn with_msg<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        self.err().with_msg(msg)
    }
}

impl From<signature::Error> for NetdocErrorSource {
    fn from(err: signature::Error) -> Self {
        NetdocErrorSource::Signature(Arc::new(err))
    }
}

/// An error that occurred while parsing a directory object of some kind.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Error {
    /// What kind of error occurred?
    pub(crate) kind: NetdocErrorKind,
    /// Do we have more information about the error?>
    msg: Option<Cow<'static, str>>,
    /// Where did the error occur?
    pos: Pos,
    /// Was this caused by another error?
    source: Option<NetdocErrorSource>,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.msg == other.msg && self.pos == other.pos
    }
}

impl Error {
    /// Helper: return this error's position.
    pub(crate) fn pos(&self) -> Pos {
        self.pos
    }

    /// Return a new error based on this one, with any byte-based
    /// position mapped to some line within a string.
    #[must_use]
    pub fn within(mut self, s: &str) -> Error {
        self.pos = self.pos.within(s);
        self
    }

    /// Return a new error based on this one, with the position (if
    /// any) replaced by 'p'.
    #[must_use]
    pub fn at_pos(mut self, p: Pos) -> Error {
        self.pos = p;
        self
    }

    /// Return a new error based on this one, with the position (if
    /// replaced by 'p' if it had no position before.
    #[must_use]
    pub fn or_at_pos(mut self, p: Pos) -> Error {
        match self.pos {
            Pos::None | Pos::Unknown => {
                self.pos = p;
            }
            _ => (),
        }
        self
    }

    /// Return a new error based on this one, with the message
    /// value set to a provided static string.
    #[must_use]
    pub(crate) fn with_msg<T>(mut self, message: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        self.msg = Some(message.into());
        self
    }

    /// Return a new error based on this one, with the source-error
    /// value set to the provided error.
    #[must_use]
    pub(crate) fn with_source<T>(mut self, source: T) -> Error
    where
        T: Into<NetdocErrorSource>,
    {
        self.source = Some(source.into());
        self
    }

    /// Return the [`NetdocErrorKind`] of this error.
    pub fn netdoc_error_kind(&self) -> NetdocErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.kind, self.pos)?;
        if let Some(msg) = &self.msg {
            write!(f, ": {}", msg)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s as _)
    }
}

/// Helper: declare an Into<> implementation to automatically convert a $source
/// into an Error with kind $kind.
macro_rules! declare_into  {
    {$source:ty => $kind:ident} => {
        impl From<$source> for Error {
            fn from(source: $source) -> Error {
                Error {
                    kind: NetdocErrorKind::$kind,
                    msg: None,
                    pos: Pos::Unknown,
                    source: Some(source.into())
                }
            }
        }
    }
}

declare_into! { signature::Error => BadSignature }
declare_into! { tor_checkable::TimeValidityError => BadTimeBound }
declare_into! { tor_bytes::Error => Undecodable }
declare_into! { std::num::ParseIntError => BadArgument }
declare_into! { std::net::AddrParseError => BadArgument }
declare_into! { PolicyError => BadPolicy }

impl From<std::convert::Infallible> for Error {
    fn from(e: std::convert::Infallible) -> Error {
        match e {}
    }
}

impl From<tor_error::Bug> for Error {
    fn from(err: tor_error::Bug) -> Self {
        use tor_error::HasKind;
        let kind = match err.kind() {
            tor_error::ErrorKind::BadApiUsage => NetdocErrorKind::BadApiUsage,
            _ => NetdocErrorKind::Internal,
        };

        Error {
            kind,
            msg: None,
            pos: Pos::Unknown,
            source: Some(err.into()),
        }
    }
}

/// An error that occurs while trying to construct a network document.
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum BuildError {
    /// We were unable to build the document, probably due to an invalid
    /// argument of some kind.
    #[error("cannot build document: {0}")]
    CannotBuild(&'static str),

    /// An argument that was given as a string turned out to be unparsable.
    #[error("unable to parse argument")]
    Parse(#[from] crate::err::Error),
}
