//! Relay flags (aka Router Status Flags), eg in network status documents

use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::str::FromStr;

use thiserror::Error;

use tor_error::internal;

use super::Unknown;

/// Raw bits value for [`RelayFlags`]
pub type RelayFlagsBits = u16;

/// Router flags (aka relay flags), including, maybe, unknown ones
///
/// ### PartialEq implementation
///
/// `DocRelayFlags` implements `PartialEq`.
///
/// Two `DocRelayFlags` which both omit unknown flags (ie, contain `Unknown::Discarded`)
/// are treated as equal if they contain the same set of *known* flags.
/// This makes sense, because applications (like clients) that discard flags during netdoc
/// parsing *want* to completely ignore unknown flags, and want to have a working comparison
/// function for relay flags (eg to tell if two relays are similar enough).
///
/// Two `RelayFlags` only *one* of which retained unknown flags are treated as unequal.
/// Such a comparison is probably a bug, but panicking would be worse.
///
/// ### `FromIterator`
///
/// With `feature = "retain-unknown"`, a
/// a `DocRelayFlags` can be assembled from an iterator of `DocRelayFlag`.
/// The inputs flags are then deduplicated, and no ordering checks are done.
///
/// So the `FromIterator` impl should not be used for parsing.
#[derive(Debug, Clone, derive_more::Deref, PartialEq)]
#[non_exhaustive]
pub struct DocRelayFlags {
    /// Known flags
    ///
    /// Invariant: contains no unknown set bits.
    #[deref]
    pub known: RelayFlags,

    /// Unknown flags, if they were parsed
    ///
    /// Not sorted.
    ///
    /// It is a semantic error to include known flags here.
    /// The parsing routines will not do so.
    pub unknown: Unknown<HashSet<String>>,
}

/// Additional options for the representation of relay flags in network documents
///
/// This is a generic argument to `ParserEncoder`
/// (and will be used for the encoder too).
pub trait ReprMode: Debug + Copy {
    /// Flags that should be treated as being present when parsing
    ///
    /// Ie, they should be inferred even if they aren't actually listed in the document.
    ///
    /// But, when encoding, they should still be emitted.
    const PARSE_IMPLICIT: RelayFlags;

    /// Flags that should be treated as being present, and won't even be encoded.
    ///
    /// These are inferred when parsing, and omitted when encoding.
    ///
    /// (During parsing `ENCODE_OMIT` and `PARSE_IMPLICIT` flags are treated the same.)
    //
    // TODO if setting this to nonzero, ensure it's actually tested.
    const ENCODE_OMIT: RelayFlags;
}

/// How relay flags are represented in `s` in a consensus
#[derive(Debug, Copy, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct ConsensusRepr;

impl ReprMode for ConsensusRepr {
    const PARSE_IMPLICIT: RelayFlags =
        RelayFlags::from_flags(&[RelayFlag::Running, RelayFlag::Valid]);
    const ENCODE_OMIT: RelayFlags = RelayFlags::empty();
}

/// How relay flags are represented in `s` in a vote and a `known-flags` line
#[derive(Debug, Copy, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct NoImplicitRepr;

impl ReprMode for NoImplicitRepr {
    const PARSE_IMPLICIT: RelayFlags = RelayFlags::empty();
    const ENCODE_OMIT: RelayFlags = RelayFlags::empty();
}

/// Set of (known) router status flags
///
/// Set of [`RelayFlag`], in a cheap and compact representation.
///
/// Can contain only flags known to this implementation.
/// The underlying bits are private, so unknown flags cannot be inserted.
///
/// Does not implement `ItemValueParseable`.  Parsing (and encoding) is different in
/// different documents.  Use an appropriate parameterised [`ParserEncoder`],
/// in `#[deftly(netdoc(with))]`.
///
/// To also maybe handle unknown flags, use [`DocRelayFlags`].
///
/// <https://spec.torproject.org/dir-spec/consensus-formats.html#item:s>
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RelayFlags(RelayFlagsBits);

impl RelayFlags {
    /// Return an empty set of relay flags.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Return all relay flags known to this implementation.
    pub const fn all() -> Self {
        Self::from_flags(&ALL_FLAGS)
    }

    /// Construct a set from known flags, including in a constant expression.
    ///
    /// This can be used to define the constants in a [`ReprMode`].
    pub const fn from_flags(flags: &[RelayFlag]) -> Self {
        let mut bits = 0;
        let mut i = 0;
        while i < flags.len() {
            bits |= flags[i].bit();
            i += 1;
        }
        Self(bits)
    }

    /// Return whether this set contains `flag`.
    pub const fn contains(&self, flag: RelayFlag) -> bool {
        self.0 & flag.bit() != 0
    }

    /// Insert `flag`, returning whether it was absent before insertion.
    pub fn insert(&mut self, flag: RelayFlag) -> bool {
        let added = !self.contains(flag);
        self.0 |= flag.bit();
        added
    }

    /// Iterate over the known flags in discriminant order.
    ///
    /// The iterator owns a copy of this set and does not borrow it.
    /// Document encoders sort the flag names separately into lexical order.
    pub fn iter(&self) -> impl Iterator<Item = RelayFlag> + use<> {
        let flags = *self;
        ALL_FLAGS
            .into_iter()
            .filter(move |flag| flags.contains(*flag))
    }
}

impl Debug for RelayFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl From<RelayFlag> for RelayFlags {
    fn from(flag: RelayFlag) -> Self {
        Self::from_flags(&[flag])
    }
}

impl<T: Into<RelayFlags>> BitOr<T> for RelayFlags {
    type Output = Self;

    fn bitor(self, rhs: T) -> Self {
        Self(self.0 | rhs.into().0)
    }
}

impl<T: Into<RelayFlags>> BitOrAssign<T> for RelayFlags {
    fn bitor_assign(&mut self, rhs: T) {
        *self = *self | rhs;
    }
}

impl<T: Into<RelayFlags>> BitAnd<T> for RelayFlags {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self {
        Self(self.0 & rhs.into().0)
    }
}

impl<T: Into<RelayFlags>> BitAndAssign<T> for RelayFlags {
    fn bitand_assign(&mut self, rhs: T) {
        *self = *self & rhs;
    }
}

impl Not for RelayFlags {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0 & Self::all().0)
    }
}

/// Router status flags - one recognized directory flag on a single relay.
///
/// <https://spec.torproject.org/dir-spec/consensus-formats.html#item:s>
///
/// These flags come from a consensus directory document, and are
/// used to describe what the authorities believe about the relay.
/// If the document contained any flags that we _didn't_ recognize,
/// they are not listed in this type.
///
/// TODO SPEC: Make the terminology the same everywhere.
#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[cfg_attr(test, derive(strum::EnumIter))]
#[non_exhaustive]
pub enum RelayFlag {
    /// Is this a directory authority?
    Authority = 0,
    /// Is this relay marked as a bad exit?
    ///
    /// Bad exits can be used as intermediate relays, but not to
    /// deliver traffic.
    BadExit = 1,
    /// Is this relay marked as an exit for weighting purposes?
    Exit = 2,
    /// Is this relay considered "fast" above a certain threshold?
    Fast = 3,
    /// Is this relay suitable for use as a guard relay?
    ///
    /// Clients choose their their initial relays from among the set
    /// of Guard relays.
    Guard = 4,
    /// Does this relay participate on the onion service directory
    /// ring?
    HSDir = 5,
    /// Set if this relay is considered "middle only", not suitable to run
    /// as an exit or guard relay.
    ///
    /// Note that this flag is only used by authorities as part of
    /// the voting process; clients do not and should not act
    /// based on whether it is set.
    MiddleOnly = 6,
    /// If set, there is no consensus for the ed25519 key for this relay.
    NoEdConsensus = 7,
    /// Is this relay considered "stable" enough for long-lived circuits?
    Stable = 8,
    /// Set if the authorities are requesting a fresh descriptor for
    /// this relay.
    StaleDesc = 9,
    /// Set if this relay is currently running.
    ///
    /// This flag can appear in votes, but in consensuses, every relay
    /// is assumed to be running.
    Running = 10,
    /// Set if this relay is considered "valid" -- allowed to be on
    /// the network.
    ///
    /// This flag can appear in votes, but in consensuses, every relay
    /// is assumed to be valid.
    Valid = 11,
    /// Set if this relay supports a currently recognized version of the
    /// directory protocol.
    V2Dir = 12,
}

impl RelayFlag {
    /// Bit corresponding to this flag in a [`RelayFlags`] set.
    const fn bit(self) -> RelayFlagsBits {
        1 << self as u16
    }
}

impl<T: Into<RelayFlags>> BitOr<T> for RelayFlag {
    type Output = RelayFlags;

    fn bitor(self, rhs: T) -> RelayFlags {
        RelayFlags::from(self) | rhs
    }
}

/// Known flags in discriminant order, preserving the existing bit assignments.
// Add new RelayFlag variants here as well as in the enum above.
const ALL_FLAGS: [RelayFlag; 13] = [
    RelayFlag::Authority,
    RelayFlag::BadExit,
    RelayFlag::Exit,
    RelayFlag::Fast,
    RelayFlag::Guard,
    RelayFlag::HSDir,
    RelayFlag::MiddleOnly,
    RelayFlag::NoEdConsensus,
    RelayFlag::Stable,
    RelayFlag::StaleDesc,
    RelayFlag::Running,
    RelayFlag::Valid,
    RelayFlag::V2Dir,
];

/// Parsing helper for a relay flags line (eg `s` item in a routerdesc)
///
#[derive(Debug, Clone)]
pub struct ParserEncoder<'s, M: ReprMode> {
    /// Flags so far, including the implied ones
    flags: DocRelayFlags,

    /// The previous argument, if any
    ///
    /// Used only for checking that the arguments are sorted, as per the spec.
    prev: Option<&'s str>,

    /// The mode, which is just a type token
    repr_mode: PhantomData<M>,
}

/// Problem parsing a relay flags line
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum RelayFlagsParseError {
    /// Flags were not in lexical order by flag name
    #[error("Flags out of order")]
    OutOfOrder,
}

impl DocRelayFlags {
    /// Create a new `DocRelayFlags` with no known flags and no information about unknown flags
    pub fn new_empty_unknown_discarded() -> Self {
        DocRelayFlags {
            known: RelayFlags::default(),
            unknown: Unknown::new_discard(),
        }
    }
}

impl<'s, M: ReprMode> ParserEncoder<'s, M> {
    /// Start parsing relay flags
    ///
    /// [`RelayFlags`] ensures the mode can contain only known flags.
    pub fn new(unknown: Unknown<()>) -> Self {
        let known = M::PARSE_IMPLICIT | M::ENCODE_OMIT;
        ParserEncoder {
            flags: DocRelayFlags {
                known,
                unknown: unknown.map(|()| HashSet::new()),
            },
            prev: None,
            repr_mode: PhantomData,
        }
    }
    /// Parse the next relay flag argument
    pub fn add(&mut self, arg: &'s str) -> Result<(), RelayFlagsParseError> {
        if let Some(prev) = self.prev {
            if prev >= arg {
                // Arguments out of order.
                return Err(RelayFlagsParseError::OutOfOrder);
            }
        }
        match RelayFlag::from_str(arg) {
            Ok(fl) => self.flags.known |= fl,
            Err(_) => self.flags.unknown.with_mut_unknown(|u| {
                u.insert(arg.to_string());
            }),
        }

        self.prev = Some(arg);
        Ok(())
    }
    /// Finish parsing relay flags
    pub fn finish(self) -> DocRelayFlags {
        self.flags
    }
}

/// Old parser impl
mod parse_impl {
    use super::*;
    use crate::doc::netstatus::NetstatusKwd;
    use crate::parse::tokenize::Item;
    use crate::{Error, NetdocErrorKind as EK, Result};

    impl DocRelayFlags {
        /// Parse a relay-flags entry from an "s" line.
        pub(crate) fn from_item_consensus(item: &Item<'_, NetstatusKwd>) -> Result<DocRelayFlags> {
            if item.kwd() != NetstatusKwd::RS_S {
                return Err(
                    Error::from(internal!("Wrong keyword {:?} for S line", item.kwd()))
                        .at_pos(item.pos()),
                );
            }
            let mut flags = ParserEncoder::<ConsensusRepr>::new(Unknown::new_discard());

            for s in item.args() {
                flags
                    .add(s)
                    .map_err(|msg| EK::BadArgument.at_pos(item.pos()).with_msg(msg.to_string()))?;
            }

            Ok(flags.finish())
        }
    }
}

/// New parser impl
mod parse2_impl {
    use super::*;

    use crate::parse2::{self, ErrorProblem as EP};
    use {
        crate::encode::ItemEncoder, itertools::chain, std::collections::BTreeSet, tor_error::Bug,
    };

    impl<'s, M: ReprMode> ParserEncoder<'s, M> {
        /// Parse relay flags
        #[allow(clippy::needless_pass_by_value)] // we must match trait signature
        pub(crate) fn from_unparsed(item: parse2::UnparsedItem<'_>) -> Result<DocRelayFlags, EP> {
            item.check_no_object()?;
            let mut flags = Self::new(item.parse_options().retain_unknown_values);
            for arg in item.args_copy() {
                flags
                    .add(arg)
                    .map_err(item.invalid_argument_handler("flags"))?;
            }
            Ok(flags.finish())
        }

        /// Encode relay flags
        #[allow(clippy::unnecessary_wraps)] // we must match trait signature
        #[allow(clippy::redundant_closure)] // rust-clippy/issues#14215 |f| <&'static str>::from(f)
        pub(crate) fn write_item_value_onto(
            flags: &DocRelayFlags,
            mut out: ItemEncoder,
        ) -> Result<(), Bug> {
            let set = chain!(
                flags
                    .known
                    .iter()
                    .filter(|f| !M::ENCODE_OMIT.contains(*f))
                    .map(|f| <&'static str>::from(f)),
                flags
                    .unknown
                    .as_ref()
                    .only_known()
                    .map(|u| u.iter())
                    .into_iter()
                    .flatten()
                    .map(|s: &String| &**s),
            )
            .collect::<BTreeSet<&'_ str>>();

            for f in set {
                out = out.arg(&f);
            }
            Ok(())
        }
    }
}

#[cfg(feature = "retain-unknown")]
pub use doc_relay_flag::*;
/// One relay flag that might be known or unknown, iterators, accessors
#[cfg(feature = "retain-unknown")]
mod doc_relay_flag {
    use super::*;
    use itertools::chain;
    use std::borrow::{Borrow, Cow};
    use tor_error::Bug;

    /// One relay flag, known or unknown
    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    #[allow(clippy::exhaustive_enums)] // there are only these two possibilities
    pub enum DocRelayFlag<'s> {
        /// Known relay flag (normally found as a bit in [`RelayFlags`])
        Known(RelayFlag),

        /// Unknown relay flag (stored as strings in [`DocRelayFlags`]).
        ///
        /// It is a semantic error to represent a known flag here; see [`DocRelayFlags`].
        /// A `DocRelayFlag` from a correct `DocRelayFlags` (eg from the parsing routines)
        /// will not do so.
        Unknown(Cow<'s, str>),
    }

    impl<'s> DocRelayFlag<'s> {
        /// Un-borrow, yielding a `DocRelayFlag<'static>`
        pub fn into_static(self) -> DocRelayFlag<'static> {
            match self {
                DocRelayFlag::Known(f) => DocRelayFlag::Known(f),
                DocRelayFlag::Unknown(s) => DocRelayFlag::Unknown(s.into_owned().into()),
            }
        }
    }

    impl DocRelayFlags {
        /// Returns an iterator over all the flags, known and unknown
        ///
        /// Throws `Bug` if the `DocRelayFlags` has `Unknown::Discarded`.
        ///
        /// Contrast `RelayFlags::iter()` (available on `DocRelayFlags` via `Deref`)
        /// which yields only the known flags.
        pub fn iter_incl_unknown(&self) -> Result<impl Iterator<Item = DocRelayFlag<'_>>, Bug> {
            let unknown = self.unknown.as_ref().into_retained()?;
            Ok(chain! {
                self.known.iter().map(DocRelayFlag::Known),
                unknown.iter().map(|s| DocRelayFlag::Unknown(s.into())),
            })
        }

        /// Does this `DocRelayFlags` contain this flag?
        ///
        /// Throws `Bug` if the `DocRelayFlags` has `Unknown::Discarded`.
        ///
        /// Contrast `RelayFlags::iter()` (available on `DocRelayFlags` via `Deref`)
        /// checks if a `RelayFlag` (a known flag) is included.
        pub fn contains_incl_unknown(&self, flag: &DocRelayFlag<'_>) -> Result<bool, Bug> {
            Ok(match flag {
                DocRelayFlag::Known(k) => self.known.contains(*k),
                DocRelayFlag::Unknown(k) => self
                    .unknown
                    .as_ref()
                    .into_retained()?
                    .contains::<str>(k.borrow()),
            })
        }
    }

    impl<'s> FromIterator<DocRelayFlag<'s>> for DocRelayFlags {
        fn from_iter<I: IntoIterator<Item = DocRelayFlag<'s>>>(input: I) -> Self {
            let mut known = RelayFlags::empty();
            let mut unknown = HashSet::new();
            for i in input {
                match i {
                    DocRelayFlag::Known(f) => known.insert(f),
                    DocRelayFlag::Unknown(f) => unknown.insert(String::from(f)),
                };
            }
            DocRelayFlags {
                known,
                unknown: Unknown::Retained(unknown),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::doc::netstatus::NetstatusKwd;
    use crate::encode::NetdocEncoder;

    fn parse<M: ReprMode>(line: &str, unknown: Unknown<()>) -> DocRelayFlags {
        let mut parser = ParserEncoder::<M>::new(unknown);
        for arg in line.split_ascii_whitespace() {
            parser.add(arg).expect("valid sorted flag vector");
        }
        parser.finish()
    }

    fn encode<M: ReprMode>(flags: &DocRelayFlags) -> String {
        let mut out = NetdocEncoder::new();
        ParserEncoder::<M>::write_item_value_onto(flags, out.item(NetstatusKwd::RS_S))
            .expect("valid flag values");
        out.finish().expect("valid flag encoding")
    }

    #[test]
    fn flag_names_and_bits() {
        use strum::IntoEnumIterator as _;

        // Derive this iterator only in tests, independently of the table that
        // implements iteration, so new variants cannot silently be omitted.
        assert_eq!(RelayFlag::iter().collect::<Vec<_>>(), ALL_FLAGS);

        // These are the existing discriminants, including their nonlexical
        // ordering. In particular, Running follows StaleDesc in the bitset.
        let vectors = [
            ("Authority", RelayFlag::Authority, 0x0001),
            ("BadExit", RelayFlag::BadExit, 0x0002),
            ("Exit", RelayFlag::Exit, 0x0004),
            ("Fast", RelayFlag::Fast, 0x0008),
            ("Guard", RelayFlag::Guard, 0x0010),
            ("HSDir", RelayFlag::HSDir, 0x0020),
            ("MiddleOnly", RelayFlag::MiddleOnly, 0x0040),
            ("NoEdConsensus", RelayFlag::NoEdConsensus, 0x0080),
            ("Stable", RelayFlag::Stable, 0x0100),
            ("StaleDesc", RelayFlag::StaleDesc, 0x0200),
            ("Running", RelayFlag::Running, 0x0400),
            ("Valid", RelayFlag::Valid, 0x0800),
            ("V2Dir", RelayFlag::V2Dir, 0x1000),
        ];
        assert_eq!(RelayFlags::all().0, 0x1fff);
        assert_eq!(RelayFlags::default(), RelayFlags::empty());
        for (name, flag, bits) in vectors {
            assert_eq!(name.parse::<RelayFlag>().expect("known flag name"), flag);
            assert_eq!(<&'static str>::from(flag), name);
            let mut flags = RelayFlags::empty();
            assert!(flags.insert(flag));
            assert!(!flags.insert(flag));
            assert_eq!(flags.0, bits);
            assert_eq!(flags.iter().collect::<Vec<_>>(), [flag]);
            assert_eq!(RelayFlags::from(flag), flags);
        }
        assert_eq!(
            RelayFlags::all().iter().collect::<Vec<_>>(),
            vectors.map(|(_, flag, _)| flag),
        );
    }

    #[test]
    fn set_operations_preserve_known_bits() {
        const FLAGS: RelayFlags =
            RelayFlags::from_flags(&[RelayFlag::Guard, RelayFlag::Exit, RelayFlag::Guard]);
        assert_eq!(FLAGS, RelayFlag::Guard | RelayFlag::Exit);
        assert_eq!(RelayFlag::Fast | FLAGS, FLAGS | RelayFlag::Fast);
        let mut flags = FLAGS;
        flags |= RelayFlag::BadExit;
        flags &= !RelayFlags::from(RelayFlag::Exit);
        assert_eq!(flags, RelayFlag::Guard | RelayFlag::BadExit);

        // Every representable set has a complement containing only known
        // flags. The three unused u16 bits must never become flags.
        for bits in 0..=RelayFlags::all().0 {
            let flags = RelayFlags::from_flags(
                &ALL_FLAGS
                    .into_iter()
                    .filter(|flag| bits & flag.bit() != 0)
                    .collect::<Vec<_>>(),
            );
            assert_eq!(flags.0, bits);
            assert_eq!(flags | !flags, RelayFlags::all());
            assert_eq!(flags & !flags, RelayFlags::empty());
            assert_eq!(!!flags, flags);
        }
    }

    #[test]
    fn consensus_and_vote_representations() {
        // An s-line from the checked-in microdescriptor consensus fixture.
        let line = "Authority Exit Fast Guard HSDir Running Stable V2Dir Valid";
        assert!(
            include_str!("../../testdata/mdconsensus1.txt")
                .lines()
                .any(|s| s.strip_prefix("s ") == Some(line))
        );
        let flags = parse::<ConsensusRepr>(line, Unknown::new_discard());
        assert_eq!(encode::<ConsensusRepr>(&flags), format!("s {line}\n"));

        // Consensus Running/Valid are implied even when omitted. Votes and
        // known-flags do not imply them; encoders still emit inferred flags.
        let omitted = "Fast Guard HSDir Stable V2Dir";
        let flags = parse::<ConsensusRepr>(omitted, Unknown::new_discard());
        assert!(flags.contains(RelayFlag::Running));
        assert!(flags.contains(RelayFlag::Valid));
        assert_eq!(
            encode::<ConsensusRepr>(&flags),
            "s Fast Guard HSDir Running Stable V2Dir Valid\n",
        );
        let vote = parse::<NoImplicitRepr>(omitted, Unknown::new_discard());
        assert!(!vote.contains(RelayFlag::Running));
        assert!(!vote.contains(RelayFlag::Valid));
        assert_eq!(encode::<NoImplicitRepr>(&vote), format!("s {omitted}\n"));
        assert_eq!(
            parse::<ConsensusRepr>("", Unknown::new_discard()).known,
            RelayFlag::Running | RelayFlag::Valid,
        );
        assert_eq!(
            parse::<NoImplicitRepr>("", Unknown::new_discard()).known,
            RelayFlags::empty(),
        );
        let all = DocRelayFlags {
            known: RelayFlags::all(),
            unknown: Unknown::new_discard(),
        };
        assert_eq!(
            encode::<NoImplicitRepr>(&all),
            concat!(
                "s Authority BadExit Exit Fast Guard HSDir MiddleOnly ",
                "NoEdConsensus Running Stable StaleDesc V2Dir Valid\n",
            ),
        );
    }

    #[test]
    fn custom_representation_omits_implicit_flags() {
        #[derive(Debug, Copy, Clone)]
        struct OmitGuard;
        impl ReprMode for OmitGuard {
            const PARSE_IMPLICIT: RelayFlags = RelayFlags::from_flags(&[RelayFlag::Running]);
            const ENCODE_OMIT: RelayFlags = RelayFlags::from_flags(&[RelayFlag::Guard]);
        }

        let flags = parse::<OmitGuard>("Fast", Unknown::new_discard());
        assert_eq!(
            flags.known,
            RelayFlag::Fast | RelayFlag::Guard | RelayFlag::Running,
        );
        assert_eq!(encode::<OmitGuard>(&flags), "s Fast Running\n");
    }

    #[test]
    fn ordering_applies_to_known_and_unknown_flags() {
        for line in [
            "Guard Fast",
            "Guard Guard",
            "FutureFlag Fast",
            "Fast FutureFlag FutureFlag",
            "Valid V2Dir",
        ] {
            let mut parser = ParserEncoder::<NoImplicitRepr>::new(Unknown::new_discard());
            assert!(
                line.split_ascii_whitespace()
                    .any(|arg| matches!(parser.add(arg), Err(RelayFlagsParseError::OutOfOrder),))
            );
        }
        assert_eq!(
            parse::<NoImplicitRepr>("Fast FutureFlag Guard", Unknown::new_discard()),
            parse::<NoImplicitRepr>("Fast Guard", Unknown::new_discard()),
        );
    }

    #[cfg(feature = "retain-unknown")]
    #[test]
    fn unknown_flags_roundtrip_and_comparison() {
        let line = "Fast FutureFlag Guard ZebraFlag";
        let flags = parse::<NoImplicitRepr>(line, Unknown::Retained(()));
        assert_eq!(encode::<NoImplicitRepr>(&flags), format!("s {line}\n"));
        assert_eq!(
            flags.iter().collect::<Vec<_>>(),
            [RelayFlag::Fast, RelayFlag::Guard]
        );
        let items: Vec<_> = flags
            .iter_incl_unknown()
            .expect("unknown flags retained")
            .collect();
        assert_eq!(items.len(), 4);
        assert!(items.iter().all(|flag| {
            flags
                .contains_incl_unknown(flag)
                .expect("unknown flags retained")
        }));
        let recollected: DocRelayFlags = items.into_iter().rev().collect();
        assert_eq!(recollected, flags);
        assert_ne!(flags, parse::<NoImplicitRepr>(line, Unknown::new_discard()));
        assert_ne!(
            flags,
            parse::<NoImplicitRepr>("Fast Guard", Unknown::Retained(()))
        );
        assert!(
            parse::<NoImplicitRepr>(line, Unknown::new_discard())
                .iter_incl_unknown()
                .is_err()
        );
    }
}
