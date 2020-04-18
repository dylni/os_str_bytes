//! This crate allows interacting with the bytes stored internally by [`OsStr`]
//! and [`OsString`], without resorting to panics or data corruption for
//! invalid UTF-8. Thus, methods can be used that are already defined on
//! [`[u8]`][slice] and [`Vec<u8>`].
//!
//! Typically, the only way to losslessly construct [`OsStr`] or [`OsString`]
//! from a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which
//! requires the bytes to be valid in UTF-8. However, since this crate makes
//! conversions directly between the platform encoding and raw bytes, even some
//! strings invalid in UTF-8 can be converted.
//!
//! # Encoding
//!
//! The encoding of bytes returned or accepted by methods of this crate is
//! intentionally left unspecified. It may vary for different platforms, so
//! defining it would run contrary to the goal of generic string handling.
//! However, the following invariants will always be upheld:
//!
//! - The encoding will be compatible with UTF-8. In particular, splitting an
//!   encoded byte sequence by a UTF-8–encoded character always produces other
//!   valid byte sequences. They can be re-encoded without error using
//!   [`OsStrBytes::from_bytes`] and similar methods.
//!
//! - All characters valid in platform strings are representable. [`OsStr`] and
//!   [`OsString`] can always be losslessly reconstructed from extracted bytes.
//!
//! Note that the chosen encoding may not match how Rust stores these strings
//! internally, which is undocumented. For instance, the result of calling
//! [`OsStr::len`] will not necessarily match the number of bytes this crate
//! uses to represent the same string.
//!
//! Additionally, concatenation may yield unexpected results without a UTF-8
//! separator. If two platform strings need to be concatenated, the only safe
//! way to do so is using [`OsString::push`]. This limitation also makes it
//! undesirable to use the bytes in interchange unless absolutely necessary. If
//! the strings need to be written as output, crate [print\_bytes] can do so
//! more safely than directly writing the bytes.
//!
//! # User Input
//!
//! Traits in this crate should ideally not be used to convert byte sequences
//! that did not originate from [`OsStr`] or a related struct. The encoding
//! used by this crate is an implementation detail, so it does not make sense
//! to expose it to users.
//!
//! Crate [bstr] offers some useful alternative methods, such as
//! [`ByteSlice::to_os_str`] and [`ByteVec::into_os_string`], that are meant
//! for user input. But, they reject some byte sequences used to represent
//! valid platform strings, which would be undesirable for reliable path
//! handling. They are best used only when accepting unknown input.
//!
//! This crate is meant to help when you already have an instance of [`OsStr`]
//! and need to modify the data in a lossless way.
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
//! # Complexity
//!
//! The time complexities of methods will vary based on what functionality is
//! available for the platform. The most efficient implementation will be used,
//! but it is important to use the most applicable method. For example,
//! [`OsStringBytes::from_vec`] will be at least as efficient as
//! [`OsStringBytes::from_bytes`], but the latter should be used when only a
//! slice is available.
//!
//! # Examples
//!
//! ```
//! use std::env;
//! use std::fs;
//! # use std::io::Result;
//!
//! use os_str_bytes::OsStrBytes;
//!
//! # fn main() -> Result<()> {
//! #     mod env {
//! #         use std::ffi::OsString;
//! #
//! #         pub fn args_os() -> impl Iterator<Item = OsString> {
//! #             let mut file = super::env::temp_dir();
//! #             file.push("os_str_bytes\u{E9}.txt");
//! #             return vec![OsString::new(), file.into_os_string()]
//! #                 .into_iter();
//! #         }
//! #     }
//! #
//! for file in env::args_os().skip(1) {
//!     if file.to_bytes().first() != Some(&b'-') {
//!         let string = "hello world";
//!         fs::write(&file, string)?;
//!         assert_eq!(string, fs::read_to_string(file)?);
//!     }
//! }
//! #     Ok(())
//! # }
//! ```
//!
//! [bstr]: https://crates.io/crates/bstr
//! [`ByteSlice::to_os_str`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteSlice.html#method.to_os_str
//! [`ByteVec::into_os_string`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteVec.html#method.into_os_string
//! [`Cow`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
//! [slice]: https://doc.rust-lang.org/std/primitive.slice.html
//! [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
//! [`OsStr::len`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.len
//! [`OsStrBytes::from_bytes`]: trait.OsStrBytes.html#tymethod.from_bytes
//! [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
//! [`OsString::push`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html#method.push
//! [`OsStringBytes::from_bytes`]: trait.OsStringBytes.html#tymethod.from_bytes
//! [`OsStringBytes::from_vec`]: trait.OsStringBytes.html#tymethod.from_vec
//! [print\_bytes]: https://crates.io/crates/print_bytes
//! [`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html

#![doc(html_root_url = "https://docs.rs/os_str_bytes/*")]
#![forbid(unsafe_code)]
#![warn(unused_results)]

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::path::Path;
use std::path::PathBuf;

#[cfg(unix)]
#[path = "unix.rs"]
mod imp;
#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

/// The error that occurs when a byte sequence is not representable in the
/// platform encoding.
///
/// On Unix, this error should never occur, but [`OsStrExt`] or [`OsStringExt`]
/// should be used instead if that needs to be guaranteed.
///
/// [`OsStrExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStrExt.html
/// [`OsStringExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStringExt.html
#[derive(Debug, Eq, PartialEq)]
pub struct EncodingError(());

impl Display for EncodingError {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        "os_str_bytes: byte sequence is not representable in the platform \
         encoding"
            .fmt(formatter)
    }
}

impl Error for EncodingError {}

/// A platform agnostic variant of [`OsStrExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: index.html
/// [`OsStrExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStrExt.html
pub trait OsStrBytes: private::Sealed + ToOwned {
    /// Converts a byte slice into an equivalent platform-native string.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi::OsStr;
    /// #
    /// # use os_str_bytes::EncodingError;
    /// use os_str_bytes::OsStrBytes;
    ///
    /// # fn main() -> Result<(), EncodingError> {
    /// let string = b"foo\xED\xA0\xBDbar";
    /// assert_eq!(string.len(), OsStr::from_bytes(string)?.len());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError>;

    /// Converts a platform-native string into an equivalent byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi::OsStr;
    /// #
    /// # use os_str_bytes::EncodingError;
    /// use os_str_bytes::OsStrBytes;
    ///
    /// # fn main() -> Result<(), EncodingError> {
    /// let string = b"foo\xED\xA0\xBDbar";
    /// let os_string = OsStr::from_bytes(string)?.into_owned();
    /// assert_eq!(string, os_string.to_bytes().as_ref());
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    fn to_bytes(&self) -> Cow<'_, [u8]>;
}

impl OsStrBytes for Path {
    #[inline]
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError> {
        OsStr::from_bytes(string).map(|os_string| match os_string {
            Cow::Borrowed(os_string) => Cow::Borrowed(Self::new(os_string)),
            Cow::Owned(os_string) => Cow::Owned(os_string.into()),
        })
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.as_os_str().to_bytes()
    }
}

/// A platform agnostic variant of [`OsStringExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: index.html
/// [`OsStringExt`]: https://doc.rust-lang.org/std/os/unix/ffi/trait.OsStringExt.html
pub trait OsStringBytes: private::Sealed + Sized {
    /// Copies a byte slice into an equivalent platform-native string.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi::OsString;
    /// #
    /// # use os_str_bytes::EncodingError;
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> Result<(), EncodingError> {
    /// let string = b"foo\xED\xA0\xBDbar";
    /// assert_eq!(string.len(), OsString::from_bytes(string)?.len());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>;

    /// Converts a byte vector into an equivalent platform-native string.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi::OsString;
    /// #
    /// # use os_str_bytes::EncodingError;
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> Result<(), EncodingError> {
    /// let string = b"foo\xED\xA0\xBDbar".to_vec();
    /// assert_eq!(string.len(), OsString::from_vec(string)?.len());
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`EncodingError`]: struct.EncodingError.html
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError>;

    /// Converts a platform-native string into an equivalent byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::ffi::OsString;
    /// #
    /// # use os_str_bytes::EncodingError;
    /// use os_str_bytes::OsStringBytes;
    ///
    /// # fn main() -> Result<(), EncodingError> {
    /// let string = b"foo\xED\xA0\xBDbar".to_vec();
    /// let os_string = OsString::from_vec(string.clone())?;
    /// assert_eq!(string, os_string.into_vec());
    /// #     Ok(())
    /// # }
    /// ```
    #[must_use]
    fn into_vec(self) -> Vec<u8>;
}

impl OsStringBytes for PathBuf {
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>,
    {
        OsString::from_bytes(string).map(Into::into)
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError> {
        OsString::from_vec(string).map(Into::into)
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        self.into_os_string().into_vec()
    }
}

mod private {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    pub trait Sealed {}
    impl Sealed for OsStr {}
    impl Sealed for OsString {}
    impl Sealed for Path {}
    impl Sealed for PathBuf {}
}
