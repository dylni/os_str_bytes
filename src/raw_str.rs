use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::mem;
use std::ops::Deref;
use std::str;

use super::imp::raw;
use super::pattern::Encoder;
use super::pattern::Pattern;
use super::OsStrBytes;
use super::OsStringBytes;

#[cfg(feature = "print_bytes")]
use print_bytes::Bytes;
#[cfg(feature = "print_bytes")]
use print_bytes::ToBytes;

#[cfg(feature = "uniquote")]
use uniquote::Quote;

fn find_pattern(string: &[u8], pat: &[u8]) -> Option<usize> {
    for i in 0..=string.len().checked_sub(pat.len())? {
        if string[i..].starts_with(pat) {
            return Some(i);
        }
    }
    None
}

fn rfind_pattern(string: &[u8], pat: &[u8]) -> Option<usize> {
    for i in (pat.len()..=string.len()).rev() {
        if string[..i].ends_with(pat) {
            return Some(i - pat.len());
        }
    }
    None
}

macro_rules! impl_trim_matches {
    ( $self:ident , $pat:expr , $strip_method:ident ) => {{
        let mut encoder = $pat.__into_encoder();
        let pat = encoder.__encode();
        if pat.is_empty() {
            return $self;
        }

        let mut string = &$self.0;
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        let mut matches = 0;
        while let Some(substring) = string.$strip_method(pat) {
            string = substring;
        }
        unsafe { Self::from_raw_bytes_unchecked(string) }
    }};
}

macro_rules! impl_split_once {
    ( $self:ident , $pat:expr , $find_fn:ident ) => {{
        let mut encoder = $pat.__into_encoder();
        let pat = encoder.__encode();

        let index = $find_fn(&$self.0, pat)?;
        let prefix = &$self.0[..index];
        let suffix = &$self.0[index + pat.len()..];
        unsafe {
            Some((
                Self::from_raw_bytes_unchecked(prefix),
                Self::from_raw_bytes_unchecked(suffix),
            ))
        }
    }};
}

/// A container for the byte strings converted by [`OsStrBytes`].
///
/// This wrapper is intended to prevent violating the invariants of the
/// [unspecified encoding] used by this crate and minimize encoding
/// conversions.
///
/// Although this type is annotated with `#[repr(transparent)]`, the inner
/// representation is not stable. Transmuting between this type and any other
/// causes immediate undefined behavior.
///
/// [unspecified encoding]: super#encoding
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
#[repr(transparent)]
pub struct RawOsStr([u8]);

impl RawOsStr {
    unsafe fn from_raw_bytes_unchecked(string: &[u8]) -> &Self {
        // SAFETY: This struct has a layout that makes this operation safe.
        mem::transmute(string)
    }

    /// Converts a platform-native string into a representation that can be
    /// more easily manipulated.
    ///
    /// This method performs the necessary conversion immediately, so it can be
    /// expensive to call. It is recommended to continue using the returned
    /// instance as long as possible (instead of the original [`OsStr`]), to
    /// avoid repeated conversions.
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
    pub fn new(string: &OsStr) -> Cow<'_, Self> {
        match string.to_raw_bytes() {
            Cow::Borrowed(string) => unsafe {
                Cow::Borrowed(Self::from_raw_bytes_unchecked(string))
            },
            Cow::Owned(string) => Cow::Owned(RawOsString(string)),
        }
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// This method is much more efficient than [`RawOsStr::new`], since the
    /// [encoding] used by this crate is compatible with UTF-8.
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
    ///
    /// [encoding]: super#encoding
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn from_str(string: &str) -> &Self {
        let string = string.as_bytes();
        unsafe { Self::from_raw_bytes_unchecked(string) }
    }

    /// Returns the byte string stored by this container.
    ///
    /// The result will match what would be returned by
    /// [`OsStrBytes::to_raw_bytes`] for the same string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// assert_eq!(os_string.to_raw_bytes(), raw.as_raw_bytes());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Equivalent to [`str::contains`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.contains("oo"));
    /// assert!(!raw.contains("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.find(pat).is_some()
    }

    /// Equivalent to [`str::ends_with`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.ends_with("bar"));
    /// assert!(!raw.ends_with("foo"));
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        self.0.ends_with(pat)
    }

    /// Equivalent to [`str::ends_with`] but accepts this type for the pattern.
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.ends_with_os(RawOsStr::from_str("bar")));
    /// assert!(!raw.ends_with_os(RawOsStr::from_str("foo")));
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with_os(&self, pat: &Self) -> bool {
        raw::ends_with(&self.0, &pat.0)
    }

    /// Equivalent to [`str::find`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(Some(1), raw.find("o"));
    /// assert_eq!(None, raw.find("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn find<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        find_pattern(&self.0, pat)
    }

    /// Equivalent to [`str::is_empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// assert!(RawOsStr::from_str("").is_empty());
    /// assert!(!RawOsStr::from_str("foobar").is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the byte string stored by this container.
    ///
    /// Only the following assumptions can be made about the result:
    /// - The length of any Unicode character is the length of its UTF-8
    ///   representation (i.e., [`char::len_utf8`]).
    /// - Splitting a string at a UTF-8 boundary will return two strings with
    ///   lengths that sum to the length of the original string.
    ///
    /// This method may return a different result than would [`OsStr::len`]
    /// when called on same string, since [`OsStr`] uses an unspecified
    /// encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// assert_eq!(6, RawOsStr::from_str("foobar").raw_len());
    /// assert_eq!(0, RawOsStr::from_str("").raw_len());
    /// ```
    #[inline]
    #[must_use]
    pub fn raw_len(&self) -> usize {
        self.0.len()
    }

    /// Equivalent to [`str::rfind`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(Some(2), raw.rfind("o"));
    /// assert_eq!(None, raw.rfind("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn rfind<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        rfind_pattern(&self.0, pat)
    }

    /// Equivalent to [`str::rsplit_once`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::from_str("fo"), RawOsStr::from_str("bar"))),
    ///     raw.rsplit_once("o"),
    /// );
    /// assert_eq!(None, raw.rsplit_once("of"));
    /// ```
    #[must_use]
    pub fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        impl_split_once!(self, pat, rfind_pattern)
    }

    /// Equivalent to [`str::split_once`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::from_str("f"), RawOsStr::from_str("obar"))),
    ///     raw.split_once("o"),
    /// );
    /// assert_eq!(None, raw.split_once("of"));
    /// ```
    #[must_use]
    pub fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        impl_split_once!(self, pat, find_pattern)
    }

    /// Equivalent to [`str::starts_with`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.starts_with("foo"));
    /// assert!(!raw.starts_with("bar"));
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        self.0.starts_with(pat)
    }

    /// Equivalent to [`str::starts_with`] but accepts this type for the
    /// pattern.
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.starts_with_os(RawOsStr::from_str("foo")));
    /// assert!(!raw.starts_with_os(RawOsStr::from_str("bar")));
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with_os(&self, pat: &Self) -> bool {
        raw::starts_with(&self.0, &pat.0)
    }

    /// Equivalent to [`str::strip_prefix`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!(
    ///     Some(RawOsStr::from_str("11foo1bar111")),
    ///     raw.strip_prefix("1"),
    /// );
    /// assert_eq!(None, raw.strip_prefix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        self.0
            .strip_prefix(pat)
            .map(|x| unsafe { Self::from_raw_bytes_unchecked(x) })
    }

    /// Equivalent to [`str::strip_suffix`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!(
    ///     Some(RawOsStr::from_str("111foo1bar11")),
    ///     raw.strip_suffix("1"),
    /// );
    /// assert_eq!(None, raw.strip_suffix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let mut encoder = pat.__into_encoder();
        let pat = encoder.__encode();

        self.0
            .strip_suffix(pat)
            .map(|x| unsafe { Self::from_raw_bytes_unchecked(x) })
    }

    /// Converts this representation back to a platform-native string.
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
    /// assert_eq!(os_string, raw.to_os_str());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn to_os_str(&self) -> Cow<'_, OsStr> {
        OsStr::from_raw_bytes(&self.0).expect("invalid raw bytes")
    }

    /// Equivalent to [`OsStr::to_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::from_str(string);
    /// assert_eq!(Some(string), raw.to_str());
    /// ```
    #[inline]
    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Converts this string to the best UTF-8 representation possible.
    ///
    /// Invalid sequences will be replaced with
    /// [`char::REPLACEMENT_CHARACTER`].
    ///
    /// This method may return a different result than would
    /// [`OsStr::to_string_lossy`] when called on same string, since [`OsStr`]
    /// uses an unspecified encoding.
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
        String::from_utf8_lossy(&self.0)
    }

    /// Equivalent to [`str::trim_end_matches`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!("111foo1bar", raw.trim_end_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_end_matches("o"));
    /// ```
    #[must_use]
    pub fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        impl_trim_matches!(self, pat, strip_suffix)
    }

    /// Equivalent to [`str::trim_start_matches`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!("foo1bar111", raw.trim_start_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_start_matches("o"));
    /// ```
    #[must_use]
    pub fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        impl_trim_matches!(self, pat, strip_prefix)
    }
}

impl AsRef<Self> for RawOsStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<RawOsStr> for str {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        RawOsStr::from_str(self)
    }
}

impl AsRef<RawOsStr> for String {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        (**self).as_ref()
    }
}

impl<'a> From<&'a RawOsStr> for Cow<'a, RawOsStr> {
    #[inline]
    fn from(other: &'a RawOsStr) -> Self {
        Cow::Borrowed(other)
    }
}

#[cfg(feature = "uniquote")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "uniquote")))]
impl Quote for RawOsStr {
    #[inline]
    fn escape(&self, f: &mut uniquote::Formatter<'_>) -> uniquote::Result {
        self.0.escape(f)
    }
}

#[cfg(feature = "print_bytes")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "print_bytes")))]
impl ToBytes for RawOsStr {
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        self.0.to_bytes()
    }
}

impl ToOwned for RawOsStr {
    type Owned = RawOsString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        RawOsString(self.0.to_vec())
    }
}

/// A container for the byte strings converted by [`OsStringBytes`].
///
/// This wrapper is intended to prevent violating the invariants of the
/// [unspecified encoding] used by this crate and minimize encoding
/// conversions.
///
/// [unspecified encoding]: super#encoding
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub struct RawOsString(Vec<u8>);

impl RawOsString {
    /// Converts a platform-native string into a representation that can be
    /// more easily manipulated.
    ///
    /// For more information, see [`RawOsStr::new`].
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
    pub fn new(string: OsString) -> Self {
        Self(string.into_raw_vec())
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// This method is much more efficient than [`RawOsString::new`], since the
    /// [encoding] used by this crate is compatible with UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_string();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(string, raw);
    /// ```
    ///
    /// [encoding]: super#encoding
    #[inline]
    #[must_use]
    pub fn from_string(string: String) -> Self {
        Self(string.into_bytes())
    }

    /// Converts this representation back to a platform-native string.
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
        OsString::from_raw_vec(self.0).expect("invalid raw bytes")
    }

    /// Returns the byte string stored by this container.
    ///
    /// The result will match what would be returned by
    /// [`OsStringBytes::into_raw_vec`] for the same string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string.clone());
    /// assert_eq!(os_string.into_raw_vec(), raw.into_raw_vec());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn into_raw_vec(self) -> Vec<u8> {
        self.0
    }

    /// Equivalent to [`OsString::into_string`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_string();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(Ok(string), raw.into_string());
    /// ```
    #[inline]
    pub fn into_string(self) -> Result<String, Self> {
        String::from_utf8(self.0).map_err(|x| Self(x.into_bytes()))
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
        unsafe { RawOsStr::from_raw_bytes_unchecked(&self.0) }
    }
}

impl From<String> for RawOsString {
    #[inline]
    fn from(other: String) -> Self {
        Self::from_string(other)
    }
}

impl From<RawOsString> for Cow<'_, RawOsStr> {
    #[inline]
    fn from(other: RawOsString) -> Self {
        Cow::Owned(other)
    }
}

#[cfg(feature = "uniquote")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "uniquote")))]
impl Quote for RawOsString {
    #[inline]
    fn escape(&self, f: &mut uniquote::Formatter<'_>) -> uniquote::Result {
        (**self).escape(f)
    }
}

#[cfg(feature = "print_bytes")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "print_bytes")))]
impl ToBytes for RawOsString {
    #[inline]
    fn to_bytes(&self) -> Bytes<'_> {
        (**self).to_bytes()
    }
}

macro_rules! r#impl {
    ( $type:ty , $other_type:ty ) => {
        impl PartialEq<$other_type> for $type {
            #[inline]
            fn eq(&self, other: &$other_type) -> bool {
                let raw: &RawOsStr = self;
                let other: &RawOsStr = other.as_ref();
                raw == other
            }
        }

        impl PartialEq<$type> for $other_type {
            #[inline]
            fn eq(&self, other: &$type) -> bool {
                other == self
            }
        }
    };
}
r#impl!(RawOsStr, RawOsString);
r#impl!(&RawOsStr, RawOsString);
r#impl!(RawOsStr, str);
r#impl!(RawOsStr, String);
r#impl!(&RawOsStr, String);
r#impl!(RawOsString, str);
r#impl!(RawOsString, &str);
r#impl!(RawOsString, String);
