//! Definitions for types used with [`NetStreamProvider`](crate::NetStreamProvider).

/// Options to use when initializing a listening socket.
///
/// This may include both options that affect the listening,
/// and options that will apply to any individual accepted connection streams.
///
/// It can include options set with `setsockopt`,
/// as well as options that influence higher layers (eg, the runtime).
///
/// For established streams that are accepted from a listener,
/// you can use [`StreamOps`](crate::StreamOps) to perform additional operations
/// or to configure additional options.
#[derive(Copy, Clone, Debug, PartialEq, Eq, getset::Getters)]
#[non_exhaustive]
pub struct CommonListenOptions {
    /// Value set for `SO_SNDBUF` on the listening socket.
    #[getset(get = "pub")]
    pub(crate) send_buffer_size: Option<usize>,

    /// Value set for `SO_RCVBUF` on the listening socket.
    #[getset(get = "pub")]
    pub(crate) recv_buffer_size: Option<usize>,
}

/// Builder for [`CommonListenOptions`].
#[derive(Clone, Default)]
pub struct CommonListenOptionsBuilder {
    /// Send buffer size, or the operating system default.
    send_buffer_size: Option<usize>,
    /// Receive buffer size, or the operating system default.
    recv_buffer_size: Option<usize>,
}

impl CommonListenOptionsBuilder {
    /// Set the value for `SO_SNDBUF`, or use the operating system default.
    pub fn send_buffer_size(&mut self, value: Option<usize>) -> &mut Self {
        self.send_buffer_size = value;
        self
    }

    /// Set the value for `SO_RCVBUF`, or use the operating system default.
    pub fn recv_buffer_size(&mut self, value: Option<usize>) -> &mut Self {
        self.recv_buffer_size = value;
        self
    }

    /// Build the socket options.
    pub fn build(&self) -> CommonListenOptions {
        CommonListenOptions {
            send_buffer_size: self.send_buffer_size,
            recv_buffer_size: self.recv_buffer_size,
        }
    }
}

impl CommonListenOptions {
    /// Returns a builder for this [`CommonListenOptions`].
    pub fn builder() -> CommonListenOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for CommonListenOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {
            send_buffer_size: None,
            recv_buffer_size: None,
        }
    }
}

/// Options to use when initializing a TCP listening socket.
///
/// See [`CommonListenOptions`] for more information.
#[derive(Copy, Clone, Debug, PartialEq, Eq, getset::Getters)]
#[non_exhaustive]
pub struct TcpListenOptions {
    /// Options that are common for all socket types.
    #[getset(get = "pub")]
    pub(crate) common: CommonListenOptions,
}

/// Builder for [`TcpListenOptions`].
#[derive(Clone, Default)]
pub struct TcpListenOptionsBuilder {
    /// Builder for the options common to socket types.
    common: CommonListenOptionsBuilder,
}

impl TcpListenOptionsBuilder {
    /// Modify the options common to socket types.
    pub fn common(&mut self) -> &mut CommonListenOptionsBuilder {
        &mut self.common
    }

    /// Build the socket options.
    pub fn build(&self) -> TcpListenOptions {
        TcpListenOptions {
            common: self.common.build(),
        }
    }
}

impl TcpListenOptions {
    /// Returns a builder for this [`TcpListenOptions`].
    pub fn builder() -> TcpListenOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for TcpListenOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {
            common: CommonListenOptions::default(),
        }
    }
}

/// Options to use when initializing a unix stream listening socket.
// TODO: We should support at least the options in `CommonListenOptions`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnixListenOptions {}

/// Builder for [`UnixListenOptions`].
#[derive(Clone, Default)]
pub struct UnixListenOptionsBuilder {}

impl UnixListenOptionsBuilder {
    /// Build the socket options.
    pub fn build(&self) -> UnixListenOptions {
        UnixListenOptions {}
    }
}

impl UnixListenOptions {
    /// Returns a builder for this [`UnixListenOptions`].
    pub fn builder() -> UnixListenOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for UnixListenOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {}
    }
}

/// Options to use when connecting a socket.
///
/// This may include both options that affect the connection attempt,
/// and options that will apply to the resulting connection stream.
///
/// It can include options set with `setsockopt`,
/// as well as options that influence higher layers (eg, the runtime).
///
/// For established streams,
/// you can use [`StreamOps`](crate::StreamOps) to perform additional operations
/// or to configure additional options.
#[derive(Copy, Clone, Debug, PartialEq, Eq, getset::Getters)]
#[non_exhaustive]
pub struct CommonConnectOptions {
    /// Value set for `SO_SNDBUF` on the socket.
    #[getset(get = "pub")]
    pub(crate) send_buffer_size: Option<usize>,

    /// Value set for `SO_RCVBUF` on the socket.
    #[getset(get = "pub")]
    pub(crate) recv_buffer_size: Option<usize>,
}

/// Builder for [`CommonConnectOptions`].
#[derive(Clone, Default)]
pub struct CommonConnectOptionsBuilder {
    /// Send buffer size, or the operating system default.
    send_buffer_size: Option<usize>,
    /// Receive buffer size, or the operating system default.
    recv_buffer_size: Option<usize>,
}

impl CommonConnectOptionsBuilder {
    /// Set the value for `SO_SNDBUF`, or use the operating system default.
    pub fn send_buffer_size(&mut self, value: Option<usize>) -> &mut Self {
        self.send_buffer_size = value;
        self
    }

    /// Set the value for `SO_RCVBUF`, or use the operating system default.
    pub fn recv_buffer_size(&mut self, value: Option<usize>) -> &mut Self {
        self.recv_buffer_size = value;
        self
    }

    /// Build the socket options.
    pub fn build(&self) -> CommonConnectOptions {
        CommonConnectOptions {
            send_buffer_size: self.send_buffer_size,
            recv_buffer_size: self.recv_buffer_size,
        }
    }
}

impl CommonConnectOptions {
    /// Returns a builder for this [`CommonConnectOptions`].
    pub fn builder() -> CommonConnectOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for CommonConnectOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {
            send_buffer_size: None,
            recv_buffer_size: None,
        }
    }
}

/// Options to use when connecting a TCP socket.
///
/// See [`CommonConnectOptions`] for more information.
#[derive(Copy, Clone, Debug, PartialEq, Eq, getset::Getters)]
#[non_exhaustive]
pub struct TcpConnectOptions {
    /// Options that are common for all socket types.
    #[getset(get = "pub")]
    pub(crate) common: CommonConnectOptions,
}

/// Builder for [`TcpConnectOptions`].
#[derive(Clone, Default)]
pub struct TcpConnectOptionsBuilder {
    /// Builder for the options common to socket types.
    common: CommonConnectOptionsBuilder,
}

impl TcpConnectOptionsBuilder {
    /// Modify the options common to socket types.
    pub fn common(&mut self) -> &mut CommonConnectOptionsBuilder {
        &mut self.common
    }

    /// Build the socket options.
    pub fn build(&self) -> TcpConnectOptions {
        TcpConnectOptions {
            common: self.common.build(),
        }
    }
}

impl TcpConnectOptions {
    /// Returns a builder for this [`TcpConnectOptions`].
    pub fn builder() -> TcpConnectOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for TcpConnectOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {
            common: CommonConnectOptions::default(),
        }
    }
}

/// Options to use when connecting a unix stream socket.
// TODO: We should support at least the options in `CommonConnectOptions`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnixConnectOptions {}

/// Builder for [`UnixConnectOptions`].
#[derive(Clone, Default)]
pub struct UnixConnectOptionsBuilder {}

impl UnixConnectOptionsBuilder {
    /// Build the socket options.
    pub fn build(&self) -> UnixConnectOptions {
        UnixConnectOptions {}
    }
}

impl UnixConnectOptions {
    /// Returns a builder for this [`UnixConnectOptions`].
    pub fn builder() -> UnixConnectOptionsBuilder {
        Default::default()
    }
}

// We want to make sure that the defaults are set to the defaults that the builder uses.
#[allow(clippy::derivable_impls)]
impl Default for UnixConnectOptions {
    fn default() -> Self {
        // This needs to match the result of `Self::builder().build()`,
        // which is tested by the `builder_defaults()` test below.
        Self {}
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
    fn builder_clones_keep_independent_socket_options() {
        const SEND_BUFFER: usize = 65_536;
        const RECV_BUFFER: usize = 32_768;

        macro_rules! check {
            ($type:ty) => {
                let mut builder = <$type>::builder();
                builder
                    .common()
                    .send_buffer_size(Some(SEND_BUFFER))
                    .recv_buffer_size(Some(RECV_BUFFER));
                let snapshot = builder.clone();
                builder
                    .common()
                    .send_buffer_size(None)
                    .recv_buffer_size(Some(0));

                for _ in 0..2 {
                    let original = snapshot.build();
                    assert_eq!(original.common().send_buffer_size(), &Some(SEND_BUFFER));
                    assert_eq!(original.common().recv_buffer_size(), &Some(RECV_BUFFER));
                    let changed = builder.build();
                    assert_eq!(changed.common().send_buffer_size(), &None);
                    assert_eq!(changed.common().recv_buffer_size(), &Some(0));
                }
            };
        }

        check!(TcpListenOptions);
        check!(TcpConnectOptions);
    }

    #[test]
    fn builder_defaults() {
        // Ensure that the builder default matches the type's default.
        macro_rules! check {
            ($type:tt) => {
                assert_eq!($type::builder().build(), $type::default());
            };
        }

        check!(CommonListenOptions);
        check!(TcpListenOptions);
        check!(UnixListenOptions);

        check!(CommonConnectOptions);
        check!(TcpConnectOptions);
        check!(UnixConnectOptions);
    }
}
