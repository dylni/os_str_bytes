//! This crate provides additional functionality for [`OsStr`] and
//! [`OsString`], without resorting to panics or corruption for invalid UTF-8.
//! Thus, familiar methods from [`str`] and [`String`] can be used.
//!
//! # Usage
//!
//! The most important trait included is [`OsStrBytesExt`], which provides
//! methods analagous to those of [`str`] but for [`OsStr`]. These methods will
//! never panic for invalid UTF-8 in a platform string, so they can be used to
//! manipulate [`OsStr`] values with the same simplicity possible for [`str`].
//!
//! Additionally, the following wrappers are provided. They are primarily
//! legacy types from when this crate needed to perform more frequent encoding
//! conversions. However, they may be useful for their trait implementations.
//! - [`RawOsStr`] is a wrapper for [`OsStr`].
//! - [`RawOsString`] is a wrapper for [`OsString`].
//!
//! # User Input
//!
//! Most methods in this crate should not be used to convert byte sequences
//! that did not originate from [`OsStr`] or a related struct. The encoding
//! used by this crate is an implementation detail, so it does not make sense
//! to expose it to users.
//!
//! For user input with an unknown encoding similar to UTF-8, use the following
//! IO-safe methods, which avoid errors when [writing to streams on
//! Windows][windows_considerations]. These methods will not accept or return
//! byte sequences that are invalid for input and output streams. Therefore,
//! they can be used to convert between bytes strings exposed to users and
//! platform strings.
//! - [`OsStrBytes::from_io_bytes`]
//! - [`OsStrBytes::to_io_bytes`]
//! - [`OsStrBytes::to_io_bytes_lossy`]
//! - [`OsStringBytes::from_io_vec`]
//! - [`OsStringBytes::into_io_vec`]
//! - [`OsStringBytes::into_io_vec_lossy`]
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file.
//!
//! ### Default Features
//!
//! - **memchr** -
//!   Changes the implementation to use crate [memchr] for better performance.
//!   This feature is useless when the "raw\_os\_str" feature is disabled.
//!
//!   For more information, see [`OsStrBytesExt`][memchr_complexity].
//!
//! - **raw\_os\_str** -
//!   Provides:
//!   - [`iter`]
//!   - [`NonUnicodeOsStr`]
//!   - [`OsStrBytesExt`]
//!   - [`Pattern`]
//!   - [`RawOsStr`]
//!   - [`RawOsStrCow`]
//!   - [`RawOsString`]
//!
//! ### Optional Features
//!
//! - **checked\_conversions** -
//!   Provides:
//!   - [`EncodingError`]
//!   - [`OsStrBytes::from_raw_bytes`]
//!   - [`OsStringBytes::from_raw_vec`]
//!   - [`RawOsStr::cow_from_raw_bytes`]
//!   - [`RawOsString::from_raw_vec`]
//!
//!   Because this feature should not be used in libraries, the
//!   "OS\_STR\_BYTES\_CHECKED\_CONVERSIONS" environment variable must be
//!   defined during compilation.
//!
//! - **conversions** -
//!   Provides methods that require encoding conversion and may be expensive:
//!   - [`OsStrBytesExt::ends_with_os`]
//!   - [`OsStrBytesExt::starts_with_os`]
//!   - [`RawOsStr::assert_cow_from_raw_bytes`]
//!   - [`RawOsStr::ends_with_os`]
//!   - [`RawOsStr::starts_with_os`]
//!   - [`RawOsStr::to_raw_bytes`]
//!   - [`RawOsString::assert_from_raw_vec`]
//!   - [`RawOsString::into_raw_vec`]
//!   - [`OsStrBytes::assert_from_raw_bytes`]
//!   - [`OsStrBytes::to_raw_bytes`]
//!   - [`OsStringBytes::assert_from_raw_vec`]
//!   - [`OsStringBytes::into_raw_vec`]
//!
//!   For more information, see [Encoding Conversions].
//!
//! # Implementation
//!
//! Some methods return [`Cow`] to account for platform differences. However,
//! no guarantee is made that the same variant of that enum will always be
//! returned for the same platform. Whichever can be constructed most
//! efficiently will be returned.
//!
//! All traits are [sealed], meaning that they can only be implemented by this
//! crate. Otherwise, backward compatibility would be more difficult to
//! maintain for new features.
//!
//! # Encoding Conversions
//!
//! Methods provided by the "conversions" feature use an intentionally
//! unspecified encoding. It may vary for different platforms, so defining it
//! would run contrary to the goal of generic string handling. However, the
//! following invariants will always be upheld:
//!
//! - The encoding will be compatible with UTF-8. In particular, splitting an
//!   encoded byte sequence by a UTF-8&ndash;encoded character always produces
//!   other valid byte sequences. They can be re-encoded without error using
//!   [`RawOsString::into_os_string`] and similar methods.
//!
//! - All characters valid in platform strings are representable. [`OsStr`] and
//!   [`OsString`] can always be losslessly reconstructed from extracted bytes.
//!
//! Note that the chosen encoding may not match how [`OsStr`] stores these
//! strings internally, which is undocumented. For instance, the result of
//! calling [`OsStr::len`] will not necessarily match the number of bytes this
//! crate uses to represent the same string. However, unlike the encoding used
//! by [`OsStr`], the encoding used by this crate can be validated safely using
//! the following methods:
//! - [`OsStrBytes::assert_from_raw_bytes`]
//! - [`RawOsStr::assert_cow_from_raw_bytes`]
//! - [`RawOsString::assert_from_raw_vec`]
//!
//! Concatenation may yield unexpected results without a UTF-8 separator. If
//! two platform strings need to be concatenated, the only safe way to do so is
//! using [`OsString::push`]. This limitation also makes it undesirable to use
//! the bytes in interchange.
//!
//! Since this encoding can change between versions and platforms, it should
//! not be used for storage. The standard library provides implementations of
//! [`OsStrExt`] and [`OsStringExt`] for various platforms, which should be
//! preferred for that use case.
//!
//! # Related Crates
//!
//! - [print\_bytes] -
//!   Used to print byte and platform strings as losslessly as possible.
//!
//! - [uniquote] -
//!   Used to display paths using escapes instead of replacement characters.
//!
//! # Examples
//!
//! ```
//! # use std::io;
//! #
//! # #[cfg(feature = "raw_os_str")]
//! # {
//! # #[cfg(any())]
//! use std::env;
//! use std::fs;
//! # use std::path::Path;
//!
//! use os_str_bytes::OsStrBytesExt;
//!
//! # mod env {
//! #   use std::ffi::OsString;
//! #   use std::iter;
//! #
//! #   use tempfile::NamedTempFile;
//! #
//! #   pub(super) fn args_os() -> impl Iterator<Item = OsString> {
//! #       let file = NamedTempFile::with_prefix("os_str_bytes_").unwrap();
//! #       iter::from_fn(move || Some(file.path().as_os_str().to_owned()))
//! #           .take(2)
//! #   }
//! # }
//! #
//! for file in env::args_os().skip(1) {
//!     if !file.starts_with('-') {
//!         let string = "Hello, world!";
//! #       assert!(Path::new(&file).exists());
//!         fs::write(&file, string)?;
//!         assert_eq!(string, fs::read_to_string(file)?);
//!     }
//! }
//! # }
//! #
//! # Ok::<_, io::Error>(())
//! ```
//!
//! [Encoding Conversions]: #encoding-conversions
//! [memchr]: https://crates.io/crates/memchr
//! [memchr_complexity]: OsStrBytesExt#complexity
//! [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
//! [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
//! [print\_bytes]: https://crates.io/crates/print_bytes
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
//! [uniquote]: https://crates.io/crates/uniquote
//! [windows_considerations]: https://doc.rust-lang.org/std/io/struct.Stdout.html#note-windows-portability-considerations

// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
// https://github.com/dylni/os_str_bytes/issues/2
#![cfg_attr(os_str_bytes_docs_rs, feature(doc_cfg))]
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![cfg_attr(target_os = "uefi", feature(uefi_std))]
#![warn(unused_results)]

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

macro_rules! if_checked_conversions {
    ( $($item:item)+ ) => {
    $(
        #[cfg(feature = "checked_conversions")]
        $item
    )+
    };
}

if_checked_conversions! {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Display;
    use std::fmt::Formatter;
    use std::result;
}

#[cfg(not(os_str_bytes_docs_rs))]
if_checked_conversions! {
    const _: &str = env!(
        "OS_STR_BYTES_CHECKED_CONVERSIONS",
        "The 'OS_STR_BYTES_CHECKED_CONVERSIONS' environment variable must be \
         defined to use the 'checked_conversions' feature.",
    );
}

#[cfg(all(feature = "memchr", not(feature = "raw_os_str")))]
const _: &str = env!(
    "__OS_STR_BYTES_CI",
    concat!(
        "The 'memchr' feature is useless when 'raw_os_str' is disabled; it \
         should be disabled too.",
    ),
);

macro_rules! if_conversions {
    ( $($item:item)+ ) => {
    $(
        #[cfg(feature = "conversions")]
        $item
    )+
    };
}

if_conversions! {
    macro_rules! expect_encoded {
        ( $result:expr ) => {
            $result.expect("invalid raw bytes")
        };
    }
}

macro_rules! if_raw_str {
    ( $($item:item)+ ) => {
    $(
        #[cfg(feature = "raw_os_str")]
        $item
    )+
    };
}

#[cfg_attr(
    all(target_family = "wasm", target_os = "unknown"),
    path = "wasm/mod.rs"
)]
#[cfg_attr(any(target_os = "uefi", windows), path = "windows/mod.rs")]
#[cfg_attr(
    not(any(
        all(target_family = "wasm", target_os = "unknown"),
        target_os = "uefi",
        windows,
    )),
    path = "common/mod.rs"
)]
mod imp;
use imp::convert_io;

if_conversions! {
    use imp::convert;
}

#[cfg(any(
    all(feature = "conversions", any(target_os = "uefi", windows)),
    feature = "raw_os_str",
))]
mod util;

if_raw_str! {
    mod ext;
    pub use ext::NonUnicodeOsStr;
    pub use ext::OsStrBytesExt;

    pub mod iter;

    mod pattern;
    pub use pattern::Pattern;

    mod raw_str;
    pub use raw_str::RawOsStr;
    pub use raw_str::RawOsStrCow;
    pub use raw_str::RawOsString;
}

if_checked_conversions! {
    /// The error that occurs when a byte sequence is not representable in the
    /// platform encoding.
    ///
    /// [`Result::unwrap`] should almost always be called on results containing
    /// this error. It should be known whether or not byte sequences are
    /// properly encoded for the platform, since [the module-level
    /// documentation][encoding] discourages using encoded bytes in
    /// interchange. Results are returned primarily to make panicking behavior
    /// explicit.
    ///
    /// On Unix, this error is never returned, but [`OsStrExt`] or
    /// [`OsStringExt`] should be used instead if that needs to be guaranteed.
    ///
    /// [encoding]: self#encoding-conversions
    /// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
    /// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
    /// [`Result::unwrap`]: ::std::result::Result::unwrap
    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(
        os_str_bytes_docs_rs,
        doc(cfg(feature = "checked_conversions"))
    )]
    pub struct EncodingError(convert::EncodingError);

    impl Display for EncodingError {
        #[inline]
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl Error for EncodingError {}
}

if_checked_conversions! {
    type Result<T> = result::Result<T, EncodingError>;
}

if_conversions! {
    fn from_raw_bytes<'a, S>(string: S) -> convert::Result<Cow<'a, OsStr>>
    where
        S: Into<Cow<'a, [u8]>>,
    {
        match string.into() {
            Cow::Borrowed(string) => convert::os_str_from_bytes(string),
            Cow::Owned(string) => {
                convert::os_string_from_vec(string).map(Cow::Owned)
            }
        }
    }
}

if_conversions! {
    fn cow_os_str_into_path(string: Cow<'_, OsStr>) -> Cow<'_, Path> {
        match string {
            Cow::Borrowed(string) => Cow::Borrowed(Path::new(string)),
            Cow::Owned(string) => Cow::Owned(string.into()),
        }
    }
}

/// A platform agnostic variant of [`OsStrExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: self
/// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
pub trait OsStrBytes: private::Sealed + ToOwned {
    if_conversions! {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// # Panics
        ///
        /// Panics if the string is not valid for the [unspecified encoding]
        /// used by this crate.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsStr;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStrBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.to_raw_bytes();
        /// assert_eq!(os_string, OsStr::assert_from_raw_bytes(os_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: self#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>;
    }

    /// Converts a byte string into an equivalent platform-native string, if it
    /// is [IO-safe].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use std::io;
    /// use std::io::Read;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// let mut io_string = Vec::new();
    /// let _ = io::stdin().read_to_end(&mut io_string)?;
    /// let os_string = OsStr::from_io_bytes(&io_string).ok_or_else(|| {
    ///     io::Error::new(io::ErrorKind::InvalidInput, "invalid input")
    /// })?;
    /// println!("{:?}", os_string);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    #[must_use]
    fn from_io_bytes(string: &[u8]) -> Option<&Self>;

    if_checked_conversions! {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// [`assert_from_raw_bytes`] should almost always be used instead. For
        /// more information, see [`EncodingError`].
        ///
        /// # Errors
        ///
        /// See documentation for [`EncodingError`].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsStr;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStrBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.to_raw_bytes();
        /// assert_eq!(os_string, OsStr::from_raw_bytes(os_bytes).unwrap());
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_from_raw_bytes`]: Self::assert_from_raw_bytes
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
        where
            S: Into<Cow<'a, [u8]>>;
    }

    /// Converts a platform-native string into an equivalent byte string, if it
    /// is [IO-safe].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::io;
    /// use std::io::Write;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let io_string = os_string.to_io_bytes().ok_or_else(|| {
    ///     io::Error::new(io::ErrorKind::InvalidInput, "invalid input")
    /// })?;
    /// io::stdout().write_all(io_string)?;
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    #[must_use]
    fn to_io_bytes(&self) -> Option<&'_ [u8]>;

    /// Converts a platform-native string into an equivalent byte string.
    ///
    /// If the string is not [IO-safe], invalid characters will be replaced
    /// with [`REPLACEMENT_CHARACTER`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::io;
    /// use std::io::Write;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let io_string = os_string.to_io_bytes_lossy();
    /// io::stdout().write_all(&io_string)?;
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    /// [`REPLACEMENT_CHARACTER`]: char::REPLACEMENT_CHARACTER
    #[must_use]
    fn to_io_bytes_lossy(&self) -> Cow<'_, [u8]>;

    if_conversions! {
        /// Converts a platform-native string into an equivalent byte string.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsStr;
        ///
        /// use os_str_bytes::OsStrBytes;
        ///
        /// let string = "foobar";
        /// let os_string = OsStr::new(string);
        /// assert_eq!(string.as_bytes(), &*os_string.to_raw_bytes());
        /// ```
        ///
        /// [unspecified encoding]: self#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[must_use]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]>;
    }
}

impl OsStrBytes for OsStr {
    if_conversions! {
        #[inline]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            expect_encoded!(from_raw_bytes(string))
        }
    }

    #[inline]
    fn from_io_bytes(string: &[u8]) -> Option<&Self> {
        convert_io::os_str_from_bytes(string)
    }

    if_checked_conversions! {
        #[inline]
        fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            from_raw_bytes(string).map_err(EncodingError)
        }
    }

    #[inline]
    fn to_io_bytes(&self) -> Option<&'_ [u8]> {
        convert_io::os_str_to_bytes(self)
    }

    #[inline]
    fn to_io_bytes_lossy(&self) -> Cow<'_, [u8]> {
        convert_io::os_str_to_bytes_lossy(self)
    }

    if_conversions! {
        #[inline]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
            convert::os_str_to_bytes(self)
        }
    }
}

impl OsStrBytes for Path {
    if_conversions! {
        #[inline]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            cow_os_str_into_path(OsStr::assert_from_raw_bytes(string))
        }
    }

    #[inline]
    fn from_io_bytes(string: &[u8]) -> Option<&Self> {
        OsStr::from_io_bytes(string).map(Self::new)
    }

    if_checked_conversions! {
        #[inline]
        fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            OsStr::from_raw_bytes(string).map(cow_os_str_into_path)
        }
    }

    #[inline]
    fn to_io_bytes(&self) -> Option<&'_ [u8]> {
        self.as_os_str().to_io_bytes()
    }

    #[inline]
    fn to_io_bytes_lossy(&self) -> Cow<'_, [u8]> {
        self.as_os_str().to_io_bytes_lossy()
    }

    if_conversions! {
        #[inline]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
            self.as_os_str().to_raw_bytes()
        }
    }
}

/// A platform agnostic variant of [`OsStringExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: self
/// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
pub trait OsStringBytes: private::Sealed + Sized {
    if_conversions! {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// # Panics
        ///
        /// Panics if the string is not valid for the [unspecified encoding]
        /// used by this crate.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsString;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStringBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.clone().into_raw_vec();
        /// assert_eq!(os_string, OsString::assert_from_raw_vec(os_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: self#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self;
    }

    /// Converts a byte string into an equivalent platform-native string, if it
    /// is [IO-safe].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsString;
    /// use std::io;
    /// use std::io::Read;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// let mut io_string = Vec::new();
    /// let _ = io::stdin().read_to_end(&mut io_string)?;
    /// let os_string = OsString::from_io_vec(io_string).ok_or_else(|| {
    ///     io::Error::new(io::ErrorKind::InvalidInput, "invalid input")
    /// })?;
    /// println!("{:?}", os_string);
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    #[must_use]
    fn from_io_vec(string: Vec<u8>) -> Option<Self>;

    if_checked_conversions! {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// [`assert_from_raw_vec`] should almost always be used instead. For
        /// more information, see [`EncodingError`].
        ///
        /// # Errors
        ///
        /// See documentation for [`EncodingError`].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsString;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStringBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.clone().into_raw_vec();
        /// assert_eq!(
        ///     os_string,
        ///     OsString::from_raw_vec(os_bytes).unwrap(),
        /// );
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_from_raw_vec`]: Self::assert_from_raw_vec
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        fn from_raw_vec(string: Vec<u8>) -> Result<Self>;
    }

    /// Converts a platform-native string into an equivalent byte string, if it
    /// is [IO-safe].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::io;
    /// use std::io::Write;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let io_string = os_string.into_io_vec().ok_or_else(|| {
    ///     io::Error::new(io::ErrorKind::InvalidInput, "invalid input")
    /// })?;
    /// io::stdout().write_all(&io_string)?;
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    #[must_use]
    fn into_io_vec(self) -> Option<Vec<u8>>;

    /// Converts a platform-native string into an equivalent byte string.
    ///
    /// If the string is not [IO-safe], invalid characters will be replaced
    /// with [`REPLACEMENT_CHARACTER`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::io;
    /// use std::io::Write;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let io_string = os_string.into_io_vec_lossy();
    /// io::stdout().write_all(&io_string)?;
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [IO-safe]: self#user-input
    /// [`REPLACEMENT_CHARACTER`]: char::REPLACEMENT_CHARACTER
    #[must_use]
    fn into_io_vec_lossy(self) -> Vec<u8>;

    if_conversions! {
        /// Converts a platform-native string into an equivalent byte string.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsString;
        ///
        /// use os_str_bytes::OsStringBytes;
        ///
        /// let string = "foobar".to_owned();
        /// let os_string: OsString = string.clone().into();
        /// assert_eq!(string.into_bytes(), os_string.into_raw_vec());
        /// ```
        ///
        /// [unspecified encoding]: self#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[must_use]
        fn into_raw_vec(self) -> Vec<u8>;
    }
}

impl OsStringBytes for OsString {
    if_conversions! {
        #[inline]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self {
            expect_encoded!(convert::os_string_from_vec(string))
        }
    }

    if_checked_conversions! {
        #[inline]
        fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            convert::os_string_from_vec(string).map_err(EncodingError)
        }
    }

    #[inline]
    fn from_io_vec(string: Vec<u8>) -> Option<Self> {
        convert_io::os_string_from_vec(string)
    }

    #[inline]
    fn into_io_vec(self) -> Option<Vec<u8>> {
        convert_io::os_string_into_vec(self)
    }

    #[inline]
    fn into_io_vec_lossy(self) -> Vec<u8> {
        convert_io::os_string_into_vec_lossy(self)
    }

    if_conversions! {
        #[inline]
        fn into_raw_vec(self) -> Vec<u8> {
            convert::os_string_into_vec(self)
        }
    }
}

impl OsStringBytes for PathBuf {
    if_conversions! {
        #[inline]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self {
            OsString::assert_from_raw_vec(string).into()
        }
    }

    if_checked_conversions! {
        #[inline]
        fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            OsString::from_raw_vec(string).map(Into::into)
        }
    }

    #[inline]
    fn from_io_vec(string: Vec<u8>) -> Option<Self> {
        OsString::from_io_vec(string).map(Into::into)
    }

    #[inline]
    fn into_io_vec(self) -> Option<Vec<u8>> {
        self.into_os_string().into_io_vec()
    }

    #[inline]
    fn into_io_vec_lossy(self) -> Vec<u8> {
        self.into_os_string().into_io_vec_lossy()
    }

    if_conversions! {
        #[inline]
        fn into_raw_vec(self) -> Vec<u8> {
            self.into_os_string().into_raw_vec()
        }
    }
}

mod private {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    if_raw_str! {
        use std::borrow::Cow;

        use super::RawOsStr;
    }

    pub trait Sealed {}

    impl Sealed for char {}
    impl Sealed for OsStr {}
    impl Sealed for OsString {}
    impl Sealed for Path {}
    impl Sealed for PathBuf {}
    impl Sealed for &str {}
    impl Sealed for &String {}

    if_raw_str! {
        impl Sealed for Cow<'_, RawOsStr> {}
    }
}
