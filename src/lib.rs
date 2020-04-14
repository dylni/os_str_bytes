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
//! # Implementation
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
//! # Related Crates
//!
//! - [print_bytes] -
//!   Assists in writing the stored bytes to an output stream, since some
//!   terminals require unicode. Internally, it uses this crate for some of its
//!   conversions.
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
//! [assumption]: https://github.com/rust-lang/rust/blob/49c68bd53f90e375bfb3cbba8c1c67a9e0adb9c0/src/libstd/sys_common/wtf8.rs#L204
//! [`char::from_u32_unchecked`]: https://doc.rust-lang.org/std/char/fn.from_u32_unchecked.html
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
//! [slice]: https://doc.rust-lang.org/std/primitive.slice.html
//! [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
//! [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
//! [`OsStringBytes::from_bytes`]: trait.OsStringBytes.html#tymethod.from_bytes
//! [`OsStringBytes::from_vec`]: trait.OsStringBytes.html#tymethod.from_vec
//! [print_bytes]: https://crates.io/crates/print_bytes
//! [`u32`]: https://doc.rust-lang.org/std/primitive.u32.html
//! [`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html

#![doc(html_root_url = "https://docs.rs/os_str_bytes/*")]
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
        "OsStr Bytes: byte sequence is not representable in the platform \
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
    /// Converts a byte slice into an equivalent platform-native string
    /// reference.
    ///
    /// This method returns [`Cow<Self>`] to account for platform differences.
    /// However, no guarantee is made that the same variant of that enum will
    /// always be returned for the same platform. Whichever can be constructed
    /// most efficiently will be returned.
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
    /// [`Cow<Self>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError>;

    /// The unsafe equivalent of [`from_bytes`].
    ///
    /// More information is given in that method's documentation.
    ///
    /// # Safety
    ///
    /// This method is unsafe, because it does not check that the bytes passed
    /// are representable in the platform encoding. If this constraint is
    /// violated, it may cause memory unsafety issues with future uses of this
    /// string, as the rest of the standard library assumes that [`OsStr`] and
    /// [`OsString`] will be usable for the platform. However, the most likely
    /// issue is that the data gets corrupted.
    ///
    /// [`from_bytes`]: #tymethod.from_bytes
    /// [`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
    /// [`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
    #[must_use]
    unsafe fn from_bytes_unchecked(string: &[u8]) -> Cow<'_, Self>;

    /// Converts the internal byte representation into a byte slice.
    ///
    /// For more information, see [`from_bytes`].
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
    ///
    /// [`from_bytes`]: #tymethod.from_bytes
    #[must_use]
    fn to_bytes(&self) -> Cow<'_, [u8]>;
}

fn os_str_into_path(os_string: Cow<'_, OsStr>) -> Cow<'_, Path> {
    match os_string {
        Cow::Borrowed(os_string) => Cow::Borrowed(Path::new(os_string)),
        Cow::Owned(os_string) => Cow::Owned(os_string.into()),
    }
}

impl OsStrBytes for Path {
    #[inline]
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError> {
        OsStr::from_bytes(string).map(os_str_into_path)
    }

    #[inline]
    unsafe fn from_bytes_unchecked(string: &[u8]) -> Cow<'_, Self> {
        os_str_into_path(OsStr::from_bytes_unchecked(string))
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
    /// Copies a byte slice into a new equivalent platform-native string.
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
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>;

    /// The unsafe equivalent of [`from_bytes`].
    ///
    /// More information is given in that method's documentation.
    ///
    /// # Safety
    ///
    /// This method is unsafe for the same reason as
    /// [`OsStrBytes::from_bytes_unchecked`].
    ///
    /// [`from_bytes`]: #tymethod.from_bytes
    /// [`OsStrBytes::from_bytes_unchecked`]: trait.OsStrBytes.html#tymethod.from_bytes_unchecked
    #[must_use]
    unsafe fn from_bytes_unchecked<TString>(string: TString) -> Self
    where
        TString: AsRef<[u8]>;

    /// Converts a byte vector into an equivalent platform-native string.
    ///
    /// Whenever possible, the conversion will be performed without copying.
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
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError>;

    /// The unsafe equivalent of [`from_vec`].
    ///
    /// More information is given in that method's documentation.
    ///
    /// # Safety
    ///
    /// This method is unsafe for the same reason as
    /// [`OsStrBytes::from_bytes_unchecked`].
    ///
    /// [`from_vec`]: #tymethod.from_vec
    /// [`OsStrBytes::from_bytes_unchecked`]: trait.OsStrBytes.html#tymethod.from_bytes_unchecked
    #[must_use]
    unsafe fn from_vec_unchecked(string: Vec<u8>) -> Self;

    /// Converts the internal byte representation into a byte vector.
    ///
    /// Whenever possible, the conversion will be performed without copying.
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
    unsafe fn from_bytes_unchecked<TString>(string: TString) -> Self
    where
        TString: AsRef<[u8]>,
    {
        OsString::from_bytes_unchecked(string).into()
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError> {
        OsString::from_vec(string).map(Into::into)
    }

    #[inline]
    unsafe fn from_vec_unchecked(string: Vec<u8>) -> Self {
        OsString::from_vec_unchecked(string).into()
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
