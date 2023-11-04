use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::mem;
use std::ops::Deref;
use std::ops::Index;
use std::result;
use std::str;

use super::ext;
use super::imp::raw;
use super::iter::RawSplit;
use super::private;
use super::OsStrBytesExt;
use super::Pattern;

if_checked_conversions! {
    use super::EncodingError;
    use super::Result;
}

if_conversions! {
    use super::convert;
}

#[allow(clippy::missing_safety_doc)]
unsafe trait TransmuteBox {
    fn transmute_box<R>(self: Box<Self>) -> Box<R>
    where
        R: ?Sized + TransmuteBox,
    {
        let value = Box::into_raw(self);
        // SAFETY: This trait is only implemented for types that can be
        // transmuted.
        unsafe { Box::from_raw(mem::transmute_copy(&value)) }
    }
}

// SAFETY: This struct has a layout that makes this operation safe.
unsafe impl TransmuteBox for RawOsStr {}
unsafe impl TransmuteBox for [u8] {}

/// A container providing additional functionality for [`OsStr`].
///
/// For more information, see [`OsStrBytesExt`].
///
/// # Safety
///
/// Although this type is annotated with `#[repr(transparent)]`, the inner
/// representation is not stable. Transmuting between this type and any other
/// causes immediate undefined behavior.
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
#[repr(transparent)]
pub struct RawOsStr([u8]);

impl RawOsStr {
    /// Wraps a platform-native string, without copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// println!("{:?}", RawOsStr::new(&os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new<S>(string: &S) -> &Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let string = string.as_ref().as_encoded_bytes();
        // SAFETY: [OsStr] prevents violating the invariants of its internal
        // encoding.
        unsafe { Self::from_encoded_bytes_unchecked(string) }
    }

    fn from_tuple<'a, 'b>(
        (prefix, suffix): (&'a OsStr, &'b OsStr),
    ) -> (&'a Self, &'b Self) {
        (Self::new(prefix), Self::new(suffix))
    }

    /// Wraps a platform-native string, without copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// println!("{:?}", RawOsStr::from_os_str(&os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[deprecated(since = "7.0.0", note = "use `new` instead")]
    #[inline]
    #[must_use]
    pub fn from_os_str(string: &OsStr) -> &Self {
        Self::new(string)
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::from_str(string);
    /// assert_eq!(string, raw);
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[deprecated(since = "7.0.0", note = "use `new` instead")]
    #[inline]
    #[must_use]
    pub fn from_str(string: &str) -> &Self {
        Self::new(string)
    }

    /// Equivalent to [`OsStr::from_encoded_bytes_unchecked`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// let raw_bytes = raw.as_encoded_bytes();
    /// assert_eq!(raw, unsafe {
    ///     RawOsStr::from_encoded_bytes_unchecked(raw_bytes)
    /// });
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[must_use]
    pub unsafe fn from_encoded_bytes_unchecked(string: &[u8]) -> &Self {
        // SAFETY: This struct has a layout that makes this operation safe.
        unsafe { mem::transmute(string) }
    }

    if_conversions! {
        fn cow_from_raw_bytes_checked(
            string: &[u8],
        ) -> convert::Result<Cow<'_, Self>> {
            convert::os_str_from_bytes(string).map(RawOsStrCow::from_os_str)
        }
    }

    if_conversions! {
        /// Converts and wraps a byte string.
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
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsStr;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsStr::new(&os_string);
        /// let raw_bytes = raw.to_raw_bytes();
        /// assert_eq!(raw, &*RawOsStr::assert_cow_from_raw_bytes(&raw_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: super#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        pub fn assert_cow_from_raw_bytes(string: &[u8]) -> Cow<'_, Self> {
            expect_encoded!(Self::cow_from_raw_bytes_checked(string))
        }
    }

    if_checked_conversions! {
        /// Converts and wraps a byte string.
        ///
        /// [`assert_cow_from_raw_bytes`] should almost always be used instead.
        /// For more information, see [`EncodingError`].
        ///
        /// # Errors
        ///
        /// See documentation for [`EncodingError`].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsStr;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsStr::new(&os_string);
        /// let raw_bytes = raw.to_raw_bytes();
        /// assert_eq!(
        ///     Ok(raw),
        ///     RawOsStr::cow_from_raw_bytes(&raw_bytes).as_deref(),
        /// );
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_cow_from_raw_bytes`]: Self::assert_cow_from_raw_bytes
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        #[inline]
        pub fn cow_from_raw_bytes(string: &[u8]) -> Result<Cow<'_, Self>> {
            Self::cow_from_raw_bytes_checked(string).map_err(EncodingError)
        }
    }

    /// Equivalent to [`OsStr::as_encoded_bytes`].
    ///
    /// The returned string will not use the [unspecified encoding]. It can
    /// only be passed to methods accepting the internal encoding of [`OsStr`],
    /// such as [`from_encoded_bytes_unchecked`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::new(string);
    /// assert_eq!(string.as_bytes(), raw.as_encoded_bytes());
    /// ```
    ///
    /// [`from_encoded_bytes_unchecked`]: Self::from_encoded_bytes_unchecked
    /// [unspecified encoding]: super#encoding-conversions
    #[inline]
    #[must_use]
    pub fn as_encoded_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Converts this representation back to a platform-native string, without
    /// copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// assert_eq!(os_string, raw.as_os_str());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_os_str(&self) -> &OsStr {
        // SAFETY: This wrapper prevents violating the invariants of the
        // internal encoding for [OsStr].
        unsafe { ext::os_str(&self.0) }
    }

    /// Equivalent to [`OsStrBytesExt::contains`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert!(raw.contains("oo"));
    /// assert!(!raw.contains("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.as_os_str().contains(pat)
    }

    /// Equivalent to [`OsStrBytesExt::ends_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert!(raw.ends_with("bar"));
    /// assert!(!raw.ends_with("foo"));
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.as_os_str().ends_with(pat)
    }

    if_conversions! {
        /// Equivalent to [`OsStrBytesExt::ends_with_os`].
        ///
        /// # Examples
        ///
        /// ```
        /// use os_str_bytes::RawOsStr;
        ///
        /// let raw = RawOsStr::new("foobar");
        /// assert!(raw.ends_with_os(RawOsStr::new("bar")));
        /// assert!(!raw.ends_with_os(RawOsStr::new("foo")));
        /// ```
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use]
        pub fn ends_with_os(&self, pat: &Self) -> bool {
            self.as_os_str().ends_with_os(pat.as_os_str())
        }
    }

    /// Equivalent to [`OsStrBytesExt::find`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert_eq!(Some(1), raw.find("o"));
    /// assert_eq!(None, raw.find("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn find<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        self.as_os_str().find(pat)
    }

    /// Equivalent to [`OsStr::is_empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// assert!(RawOsStr::new("").is_empty());
    /// assert!(!RawOsStr::new("foobar").is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_os_str().is_empty()
    }

    /// Equivalent to [`OsStrBytesExt::rfind`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert_eq!(Some(2), raw.rfind("o"));
    /// assert_eq!(None, raw.rfind("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn rfind<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        self.as_os_str().rfind(pat)
    }

    /// Equivalent to [`OsStrBytesExt::rsplit_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::new("fo"), RawOsStr::new("bar"))),
    ///     raw.rsplit_once("o"),
    /// );
    /// assert_eq!(None, raw.rsplit_once("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        self.as_os_str().rsplit_once(pat).map(Self::from_tuple)
    }

    /// Equivalent to [`OsStrBytesExt::split`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert!(raw.split("o").eq(["f", "", "bar"]));
    /// ```
    #[inline]
    #[track_caller]
    pub fn split<P>(&self, pat: P) -> RawSplit<'_, P>
    where
        P: Pattern,
    {
        RawSplit::new(self, pat)
    }

    /// Equivalent to [`OsStrBytesExt::split_at`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert_eq!(
    ///     (RawOsStr::new("fo"), RawOsStr::new("obar")),
    ///     raw.split_at(2),
    /// );
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        Self::from_tuple(self.as_os_str().split_at(mid))
    }

    /// Equivalent to [`OsStrBytesExt::split_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::new("f"), RawOsStr::new("obar"))),
    ///     raw.split_once("o"),
    /// );
    /// assert_eq!(None, raw.split_once("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        self.as_os_str().split_once(pat).map(Self::from_tuple)
    }

    /// Equivalent to [`OsStrBytesExt::starts_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert!(raw.starts_with("foo"));
    /// assert!(!raw.starts_with("bar"));
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.as_os_str().starts_with(pat)
    }

    if_conversions! {
        /// Equivalent to [`OsStrBytesExt::starts_with_os`].
        ///
        /// # Examples
        ///
        /// ```
        /// use os_str_bytes::RawOsStr;
        ///
        /// let raw = RawOsStr::new("foobar");
        /// assert!(raw.starts_with_os(RawOsStr::new("foo")));
        /// assert!(!raw.starts_with_os(RawOsStr::new("bar")));
        /// ```
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use]
        pub fn starts_with_os(&self, pat: &Self) -> bool {
            self.as_os_str().starts_with_os(pat.as_os_str())
        }
    }

    /// Equivalent to [`OsStrBytesExt::strip_prefix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("111foo1bar111");
    /// assert_eq!(Some(RawOsStr::new("11foo1bar111")), raw.strip_prefix("1"));
    /// assert_eq!(None, raw.strip_prefix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        self.as_os_str().strip_prefix(pat).map(Self::new)
    }

    /// Equivalent to [`OsStrBytesExt::strip_suffix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("111foo1bar111");
    /// assert_eq!(Some(RawOsStr::new("111foo1bar11")), raw.strip_suffix("1"));
    /// assert_eq!(None, raw.strip_suffix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        self.as_os_str().strip_suffix(pat).map(Self::new)
    }

    if_conversions! {
        /// Converts and returns the byte string stored by this container.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use os_str_bytes::RawOsStr;
        ///
        /// let string = "foobar";
        /// let raw = RawOsStr::new(string);
        /// assert_eq!(string.as_bytes(), &*raw.to_raw_bytes());
        /// ```
        ///
        /// [unspecified encoding]: super#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use]
        pub fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
            convert::os_str_to_bytes(self.as_os_str())
        }
    }

    /// Equivalent to [`OsStr::to_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::new(string);
    /// assert_eq!(Some(string), raw.to_str());
    /// ```
    #[inline]
    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        self.as_os_str().to_str()
    }

    /// Equivalent to [`OsStr::to_string_lossy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// println!("{}", raw.to_str_lossy());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn to_str_lossy(&self) -> Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    /// Equivalent to [`OsStrBytesExt::trim_end_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("111foo1bar111");
    /// assert_eq!("111foo1bar", raw.trim_end_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_end_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        Self::new(self.as_os_str().trim_end_matches(pat))
    }

    /// Equivalent to [`OsStrBytesExt::trim_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("111foo1bar111");
    /// assert_eq!("foo1bar", raw.trim_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        Self::new(self.as_os_str().trim_matches(pat))
    }

    /// Equivalent to [`OsStrBytesExt::trim_start_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("111foo1bar111");
    /// assert_eq!("foo1bar111", raw.trim_start_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_start_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        Self::new(self.as_os_str().trim_start_matches(pat))
    }
}

impl AsRef<Self> for RawOsStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<OsStr> for RawOsStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl AsRef<RawOsStr> for OsStr {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        RawOsStr::new(self)
    }
}

impl AsRef<RawOsStr> for OsString {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        (**self).as_ref()
    }
}

impl AsRef<RawOsStr> for str {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        RawOsStr::new(self)
    }
}

impl AsRef<RawOsStr> for String {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        (**self).as_ref()
    }
}

impl Default for &RawOsStr {
    #[inline]
    fn default() -> Self {
        RawOsStr::new("")
    }
}

impl<'a> From<&'a RawOsStr> for Cow<'a, RawOsStr> {
    #[inline]
    fn from(value: &'a RawOsStr) -> Self {
        Cow::Borrowed(value)
    }
}

impl From<Box<str>> for Box<RawOsStr> {
    #[inline]
    fn from(value: Box<str>) -> Self {
        value.into_boxed_bytes().transmute_box()
    }
}

impl<Idx> Index<Idx> for RawOsStr
where
    Idx: ext::SliceIndex,
{
    type Output = Self;

    #[inline]
    fn index(&self, idx: Idx) -> &Self::Output {
        Self::new(self.as_os_str().index(idx))
    }
}

impl ToOwned for RawOsStr {
    type Owned = RawOsString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        RawOsString(self.0.to_owned())
    }
}

/// Extensions to [`Cow<RawOsStr>`] for additional conversions.
///
/// [`Cow<RawOsStr>`]: Cow
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait RawOsStrCow<'a>: private::Sealed {
    /// Converts a platform-native string back to this representation, without
    /// copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    /// use os_str_bytes::RawOsStrCow;
    ///
    /// let os_string = Cow::Owned(env::current_exe()?.into_os_string());
    /// println!("{:?}", Cow::from_os_str(os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[must_use]
    fn from_os_str(string: Cow<'a, OsStr>) -> Self;

    /// Converts this representation back to a platform-native string, without
    /// copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    /// use os_str_bytes::RawOsStrCow;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = Cow::Borrowed(RawOsStr::new(&os_string));
    /// assert_eq!(os_string, raw.into_os_str());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[must_use]
    fn into_os_str(self) -> Cow<'a, OsStr>;
}

impl<'a> RawOsStrCow<'a> for Cow<'a, RawOsStr> {
    #[inline]
    fn from_os_str(string: Cow<'a, OsStr>) -> Self {
        match string {
            Cow::Borrowed(string) => Cow::Borrowed(RawOsStr::new(string)),
            Cow::Owned(string) => Cow::Owned(RawOsString::new(string)),
        }
    }

    #[inline]
    fn into_os_str(self) -> Cow<'a, OsStr> {
        match self {
            Cow::Borrowed(string) => Cow::Borrowed(string.as_os_str()),
            Cow::Owned(string) => Cow::Owned(string.into_os_string()),
        }
    }
}

/// A container for owned byte strings converted by this crate.
///
/// For more information, see [`RawOsStr`].
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub struct RawOsString(Vec<u8>);

impl RawOsString {
    /// Wraps a platform-native string, without copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// println!("{:?}", RawOsString::new(os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new<S>(string: S) -> Self
    where
        S: Into<OsString>,
    {
        Self(string.into().into_encoded_bytes())
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(string, raw);
    /// ```
    #[deprecated(since = "7.0.0", note = "use `new` instead")]
    #[inline]
    #[must_use]
    pub fn from_string(string: String) -> Self {
        Self(string.into_bytes())
    }

    /// Equivalent to [`OsString::from_encoded_bytes_unchecked`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string);
    /// let raw_bytes = raw.clone().into_encoded_vec();
    /// assert_eq!(raw, unsafe {
    ///     RawOsString::from_encoded_vec_unchecked(raw_bytes)
    /// });
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[must_use]
    pub unsafe fn from_encoded_vec_unchecked(string: Vec<u8>) -> Self {
        Self(string)
    }

    if_conversions! {
        fn from_raw_vec_checked(string: Vec<u8>) -> convert::Result<Self> {
            convert::os_string_from_vec(string).map(Self::new)
        }
    }

    if_conversions! {
        /// Wraps a byte string, without copying or encoding conversion.
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
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsString;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsString::new(os_string);
        /// let raw_bytes = raw.clone().into_raw_vec();
        /// assert_eq!(raw, RawOsString::assert_from_raw_vec(raw_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: super#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        pub fn assert_from_raw_vec(string: Vec<u8>) -> Self {
            expect_encoded!(Self::from_raw_vec_checked(string))
        }
    }

    if_checked_conversions! {
        /// Wraps a byte string, without copying or encoding conversion.
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
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsString;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsString::new(os_string);
        /// let raw_bytes = raw.clone().into_raw_vec();
        /// assert_eq!(Ok(raw), RawOsString::from_raw_vec(raw_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_from_raw_vec`]: Self::assert_from_raw_vec
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        #[inline]
        pub fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            Self::from_raw_vec_checked(string).map_err(EncodingError)
        }
    }

    /// Equivalent to [`String::clear`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let mut raw = RawOsString::new(os_string);
    /// raw.clear();
    /// assert!(raw.is_empty());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Equivalent to [`String::into_boxed_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::new(string.clone());
    /// assert_eq!(string, *raw.into_box());
    /// ```
    #[inline]
    #[must_use]
    pub fn into_box(self) -> Box<RawOsStr> {
        self.0.into_boxed_slice().transmute_box()
    }

    /// Equivalent to [`OsString::into_encoded_bytes`].
    ///
    /// The returned string will not use the [unspecified encoding]. It can
    /// only be passed to methods accepting the internal encoding of [`OsStr`],
    /// such as [`from_encoded_vec_unchecked`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::new(string.clone());
    /// assert_eq!(string.into_bytes(), raw.into_encoded_vec());
    /// ```
    ///
    /// [`from_encoded_vec_unchecked`]: Self::from_encoded_vec_unchecked
    /// [unspecified encoding]: super#encoding-conversions
    #[inline]
    #[must_use]
    pub fn into_encoded_vec(self) -> Vec<u8> {
        self.0
    }

    /// Converts this representation back to a platform-native string, without
    /// copying or encoding conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string.clone());
    /// assert_eq!(os_string, raw.into_os_string());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn into_os_string(self) -> OsString {
        // SAFETY: This wrapper prevents violating the invariants of the
        // internal encoding for [OsStr].
        unsafe { OsString::from_encoded_bytes_unchecked(self.0) }
    }

    if_conversions! {
        /// Returns the byte string stored by this container.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use os_str_bytes::RawOsString;
        ///
        /// let string = "foobar".to_owned();
        /// let raw = RawOsString::new(string.clone());
        /// assert_eq!(string.into_bytes(), raw.into_raw_vec());
        /// ```
        ///
        /// [unspecified encoding]: super#encoding-conversions
        #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
        #[inline]
        #[must_use]
        pub fn into_raw_vec(self) -> Vec<u8> {
            convert::os_string_into_vec(self.into_os_string())
        }
    }

    /// Equivalent to [`OsString::into_string`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::new(string.clone());
    /// assert_eq!(Ok(string), raw.into_string());
    /// ```
    #[inline]
    pub fn into_string(self) -> result::Result<String, Self> {
        String::from_utf8(self.0).map_err(|x| Self(x.into_bytes()))
    }

    /// Equivalent to [`String::shrink_to_fit`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let mut raw = RawOsString::new(string.clone());
    /// raw.shrink_to_fit();
    /// assert_eq!(string, raw);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    #[track_caller]
    fn check_bound(&self, index: usize) {
        ext::check_bound(self.as_os_str(), index);
    }

    /// Equivalent to [`String::split_off`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let mut raw = RawOsString::new("foobar".to_owned());
    /// assert_eq!("bar", raw.split_off(3));
    /// assert_eq!("foo", raw);
    /// ```
    ///
    /// [valid boundary]: OsStrBytesExt#indices
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn split_off(&mut self, at: usize) -> Self {
        self.check_bound(at);

        Self(self.0.split_off(at))
    }

    /// Equivalent to [`String::truncate`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let mut raw = RawOsString::new("foobar".to_owned());
    /// raw.truncate(3);
    /// assert_eq!("foo", raw);
    /// ```
    ///
    /// [valid boundary]: OsStrBytesExt#indices
    #[inline]
    #[track_caller]
    pub fn truncate(&mut self, new_len: usize) {
        self.check_bound(new_len);

        self.0.truncate(new_len);
    }
}

impl AsRef<OsStr> for RawOsString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        (**self).as_ref()
    }
}

impl AsRef<RawOsStr> for RawOsString {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        self
    }
}

impl Borrow<RawOsStr> for RawOsString {
    #[inline]
    fn borrow(&self) -> &RawOsStr {
        self
    }
}

impl Deref for RawOsString {
    type Target = RawOsStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: This wrapper prevents violating the invariants of the
        // internal encoding for [OsStr].
        unsafe { RawOsStr::from_encoded_bytes_unchecked(&self.0) }
    }
}

impl From<RawOsString> for Box<RawOsStr> {
    #[inline]
    fn from(value: RawOsString) -> Self {
        value.into_box()
    }
}

impl From<Box<RawOsStr>> for RawOsString {
    #[inline]
    fn from(value: Box<RawOsStr>) -> Self {
        Self(value.transmute_box::<[_]>().into_vec())
    }
}

impl From<RawOsString> for Cow<'_, RawOsStr> {
    #[inline]
    fn from(value: RawOsString) -> Self {
        Cow::Owned(value)
    }
}

impl From<OsString> for RawOsString {
    #[inline]
    fn from(value: OsString) -> Self {
        Self::new(value)
    }
}

impl From<RawOsString> for OsString {
    #[inline]
    fn from(value: RawOsString) -> Self {
        value.into_os_string()
    }
}

impl From<String> for RawOsString {
    #[inline]
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

struct DebugBuffer<'a>(&'a [u8]);

impl Debug for DebugBuffer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;

        let mut string = self.0;
        let mut invalid_length = 0;
        while !string.is_empty() {
            let (invalid, substring) = string.split_at(invalid_length);

            let valid = match str::from_utf8(substring) {
                Ok(valid) => {
                    string = b"";
                    valid
                }
                Err(error) => {
                    let (valid, substring) =
                        substring.split_at(error.valid_up_to());

                    let invalid_char_length =
                        error.error_len().unwrap_or_else(|| substring.len());
                    if valid.is_empty() {
                        invalid_length += invalid_char_length;
                        continue;
                    }
                    string = substring;
                    invalid_length = invalid_char_length;

                    // SAFETY: This slice was validated to be UTF-8.
                    unsafe { str::from_utf8_unchecked(valid) }
                }
            };

            // SAFETY: This substring was separated by a UTF-8 string.
            raw::debug(
                unsafe { RawOsStr::from_encoded_bytes_unchecked(invalid) },
                f,
            )?;
            Display::fmt(&valid.escape_debug(), f)?;
        }

        f.write_str("\"")
    }
}

macro_rules! r#impl {
    ( $type:ty ) => {
        impl Debug for $type {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($type))
                    .field(&DebugBuffer(&self.0))
                    .finish()
            }
        }
    };
}
r#impl!(RawOsStr);
r#impl!(RawOsString);

impl<Idx> Index<Idx> for RawOsString
where
    Idx: ext::SliceIndex,
{
    type Output = <RawOsStr as Index<Idx>>::Output;

    #[inline]
    fn index(&self, idx: Idx) -> &Self::Output {
        &(**self)[idx]
    }
}

macro_rules! r#impl {
    ( $(#[$attr:meta])* $type:ty , $other_type:ty ) => {
        $(#[$attr])*
        impl PartialEq<$other_type> for $type {
            #[inline]
            fn eq(&self, other: &$other_type) -> bool {
                let raw: &RawOsStr = self;
                let other: &RawOsStr = other.as_ref();
                raw == other
            }
        }

        $(#[$attr])*
        impl PartialEq<$type> for $other_type {
            #[inline]
            fn eq(&self, other: &$type) -> bool {
                other == self
            }
        }
    };
}
r#impl!(RawOsStr, OsStr);
r#impl!(RawOsStr, OsString);
r#impl!(RawOsStr, RawOsString);
r#impl!(RawOsStr, str);
r#impl!(RawOsStr, String);
r#impl!(&RawOsStr, OsString);
r#impl!(&RawOsStr, RawOsString);
r#impl!(&RawOsStr, String);
r#impl!(RawOsString, OsStr);
r#impl!(RawOsString, &OsStr);
r#impl!(RawOsString, OsString);
r#impl!(RawOsString, str);
r#impl!(RawOsString, &str);
r#impl!(RawOsString, String);

#[cfg(feature = "print_bytes")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "print_bytes")))]
mod print_bytes {
    use print_bytes::ByteStr;
    use print_bytes::ToBytes;
    #[cfg(windows)]
    use print_bytes::WideStr;

    #[cfg(windows)]
    use crate::imp::raw;

    use super::RawOsStr;
    use super::RawOsString;

    impl ToBytes for RawOsStr {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            self.0.to_bytes()
        }

        #[cfg(windows)]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            Some(WideStr::new(raw::encode_wide(self).collect()))
        }
    }

    impl ToBytes for RawOsString {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            (**self).to_bytes()
        }

        #[cfg(windows)]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            (**self).to_wide()
        }
    }
}

#[cfg(feature = "uniquote")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "uniquote")))]
mod uniquote {
    use uniquote::Formatter;
    use uniquote::Quote;
    use uniquote::Result;

    use crate::imp::raw;

    use super::RawOsStr;
    use super::RawOsString;

    impl Quote for RawOsStr {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            raw::uniquote::escape(self, f)
        }
    }

    impl Quote for RawOsString {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            (**self).escape(f)
        }
    }
}
