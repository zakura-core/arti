//! Error types for `tor-persist.

#![forbid(unsafe_code)] // if you remove this, enable (or write) miri tests (git grep miri)

use std::sync::Arc;

use crate::FsMistrustErrorExt as _;
use crate::slug::BadSlug;
use fs_mistrust::anon_home::PathExt as _;
use tor_basic_utils::PathExt as _;
use tor_error::{Bug, ErrorKind, into_bad_api_usage};

/// A resource that we failed to access or where we found a problem.
#[derive(Debug, Clone)]
pub(crate) enum Resource {
    /// The manager as a whole.
    Manager,
    /// A checked directory.
    Directory {
        /// The path to the directory.
        dir: std::path::PathBuf,
    },
    /// A file on disk within our checked directory.
    File {
        /// The path to the checked directory
        container: std::path::PathBuf,
        /// The path within the checked directory to the file.
        file: std::path::PathBuf,
    },
    /// Testing-only: a scratch-item in a memory-backed store.
    #[cfg(feature = "testing")]
    Temporary {
        /// The key for the scratch item
        key: String,
    },
    /// An instance state directory
    InstanceState {
        /// The path to the top-level state directory.
        state_dir: std::path::PathBuf,
        /// The instance's kind
        kind: String,
        /// The instance's identity
        identity: String,
    },
}

impl core::fmt::Display for Resource {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Manager => formatter.write_fmt(format_args!("persistent storage manager")),
            Self::Directory { dir } => {
                formatter.write_fmt(format_args!("directory {0}", dir.anonymize_home()))
            }
            Self::File { container, file } => formatter.write_fmt(format_args!(
                "{0} in {1}",
                file.display_lossy(),
                container.anonymize_home(),
            )),
            #[cfg(feature = "testing")]
            Self::Temporary { key } => {
                formatter.write_fmt(format_args!("{0} in memory-backed store", key))
            }
            Self::InstanceState {
                state_dir,
                kind,
                identity,
            } => formatter.write_fmt(format_args!(
                "instance {0:?}/{1:?} in {2}",
                kind,
                identity,
                state_dir.anonymize_home(),
            )),
        }
    }
}

/// An action that we were trying to perform when an error occurred.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Action {
    /// We were trying to load an element from the store.
    Loading,
    /// We were trying to save an element into the store.
    Storing,
    /// We were trying to remove an element from the store.
    Deleting,
    /// We were trying to acquire the lock for the store.
    Locking,
    /// We were trying to validate the storage and initialize the manager.
    Initializing,
    /// We were trying to enumerate state objects
    Enumerating,
}

impl core::fmt::Display for Action {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Loading => formatter.write_fmt(format_args!("loading persistent data")),
            Self::Storing => formatter.write_fmt(format_args!("storing persistent data")),
            Self::Deleting => formatter.write_fmt(format_args!("deleting persistent data")),
            Self::Locking => formatter.write_fmt(format_args!("acquiring lock")),
            Self::Initializing => formatter.write_fmt(format_args!("constructing storage manager")),
            Self::Enumerating => formatter.write_fmt(format_args!("enumerating instances")),
        }
    }
}

/// An underlying error manipulating persistent state.
///
/// Since these are more or less orthogonal to what we were doing and where the
/// problem was, this is a separate type.
#[derive(thiserror::Error, Debug, Clone)]
#[non_exhaustive]
pub enum ErrorSource {
    /// An IO error occurred.
    #[error("IO error")]
    IoError(#[source] Arc<std::io::Error>),

    /// Inaccessible path, or permissions were incorrect
    #[error("Problem accessing persistent state")]
    Inaccessible(#[source] fs_mistrust::Error),

    /// Inaccessible path, or permissions were incorrect
    ///
    /// This variant name is misleading - see the docs for [`fs_mistrust::Error`].
    /// Please use [`ErrorSource::Inaccessible`] instead.
    #[deprecated = "use ErrorSource::Inaccessible instead"]
    #[error("Problem accessing persistent state")]
    Permissions(#[source] fs_mistrust::Error),

    /// Tried to save without holding an exclusive lock.
    //
    // TODO This error seems to actually be sometimes used to make store a no-op.
    //      We should consider whether this is best handled as an error, but for now
    //      this seems adequate.
    #[error("Storage not locked")]
    NoLock,

    /// Problem when serializing or deserializing JSON data.
    #[error("JSON error")]
    Serde(#[from] Arc<serde_json::Error>),

    /// Another task or process holds this persistent state lock, but we need exclusive access
    #[error("State already lockedr")]
    AlreadyLocked,

    /// Programming error
    #[error("Programming error")]
    Bug(#[from] Bug),
}

impl From<BadSlug> for ErrorSource {
    fn from(bs: BadSlug) -> ErrorSource {
        into_bad_api_usage!("bad slug")(bs).into()
    }
}
/// [`BadSlug`] errors auto-convert to a [`BadApiUsage`](tor_error::ErrorKind::BadApiUsage)
///
/// (Users of `tor-persist` ought to have newtypes for user-supplied slugs,
/// and thereby avoid passing syntactically invalid slugs to `tor-persist`.)
impl From<BadSlug> for Error {
    fn from(bs: BadSlug) -> Error {
        // This metadata is approximate, but better information isn't readily available
        // and this shouldn't really happen.
        Error::new(bs, Action::Initializing, Resource::Manager)
    }
}

/// An error that occurred while manipulating persistent state.
#[derive(Clone, Debug)]
pub struct Error {
    /// The underlying error failure.
    source: ErrorSource,
    /// The action we were trying to perform
    action: Action,
    /// The resource we were trying to perform it on.
    resource: Resource,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let source = &self.source;
        let action = &self.action;
        let resource = &self.resource;
        formatter.write_fmt(format_args!(
            "{0} while {1} on {2}",
            source, action, resource
        ))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl Error {
    /// Return the underlying error source.
    pub fn source(&self) -> &ErrorSource {
        &self.source
    }

    /// Construct a new Error from its components.
    pub(crate) fn new(err: impl Into<ErrorSource>, action: Action, resource: Resource) -> Self {
        Error {
            source: err.into(),
            action,
            resource,
        }
    }
}

impl tor_error::HasKind for Error {
    #[rustfmt::skip] // the tabular layout of the `match` makes this a lot clearer
    fn kind(&self) -> ErrorKind {
        use ErrorSource as E;
        use tor_error::ErrorKind as K;
        #[allow(deprecated)]
        match &self.source {
            E::IoError(..)     => K::PersistentStateAccessFailed,
            E::Permissions(e)  => e.state_error_kind(),
            E::Inaccessible(e) => e.state_error_kind(),
            E::NoLock          => K::BadApiUsage,
            E::AlreadyLocked   => K::LocalResourceAlreadyInUse,
            E::Bug(e)          => e.kind(),
            E::Serde(..) if self.action == Action::Storing  => K::Internal,
            E::Serde(..) => K::PersistentStateCorrupted,
        }
    }
}

impl From<std::io::Error> for ErrorSource {
    fn from(e: std::io::Error) -> ErrorSource {
        ErrorSource::IoError(Arc::new(e))
    }
}

impl From<serde_json::Error> for ErrorSource {
    fn from(e: serde_json::Error) -> ErrorSource {
        ErrorSource::Serde(Arc::new(e))
    }
}

impl From<fs_mistrust::Error> for ErrorSource {
    fn from(e: fs_mistrust::Error) -> ErrorSource {
        match e {
            fs_mistrust::Error::Io { err, .. } => ErrorSource::IoError(err),
            other => ErrorSource::Inaccessible(other),
        }
    }
}

#[cfg(all(test, not(miri) /* fs-mistrust home directory lookup */))]
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
    use std::io;
    use tor_error::ErrorReport as _;

    #[test]
    fn error_display() {
        assert_eq!(
            Error::new(
                io::Error::from(io::ErrorKind::PermissionDenied),
                Action::Initializing,
                Resource::InstanceState {
                    state_dir: "/STATE_DIR".into(),
                    kind: "KIND".into(),
                    identity: "IDENTY".into(),
                }
            )
            .report()
            .to_string(),
            r#"error: IO error while constructing storage manager on instance "KIND"/"IDENTY" in /STATE_DIR: permission denied"#
        );
    }
}
