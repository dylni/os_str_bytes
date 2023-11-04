use std::ffi::OsStr;
use std::ops::Range;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeInclusive;
use std::ops::RangeTo;
use std::ops::RangeToInclusive;
use std::str;

use super::iter::Split;
use super::pattern::Encoded as EncodedPattern;
use super::util;
use super::util::MAX_UTF8_LENGTH;
use super::Pattern;

if_conversions! {
    use super::imp::raw;
    use super::OsStrBytes;
}

#[cfg(not(feature = "conversions"))]
use super::private;

fn is_boundary(string: &OsStr, index: usize) -> bool {
    let string = string.as_encoded_bytes();
    debug_assert!(index < string.len());

    if index == 0 {
        return true;
    }
    let byte = string[index];
    if byte.is_ascii() {
        return true;
    }

    if !util::is_continuation(byte) {
        let bytes = &string[index..];
        let valid = str::from_utf8(&bytes[..bytes.len().min(MAX_UTF8_LENGTH)])
            .err()
            .map(|x| x.valid_up_to() != 0)
            .unwrap_or(true);
        if valid {
            return true;
        }
    }
    let mut start = index;
    for _ in 0..MAX_UTF8_LENGTH {
        if let Some(index) = start.checked_sub(1) {
            start = index;
        } else {
            return false;
        }
        if !util::is_continuation(string[start]) {
            break;
        }
    }
    str::from_utf8(&string[start..index]).is_ok()
}

#[track_caller]
pub(super) fn check_bound(string: &OsStr, index: usize) {
    assert!(
        index >= string.as_encoded_bytes().len() || is_boundary(string, index),
        "byte index {} is not a valid boundary",
        index,
    );
}

#[cfg(feature = "memchr")]
use memchr::memmem::find;
#[cfg(feature = "memchr")]
use memchr::memmem::rfind;

#[cfg(not(feature = "memchr"))]
fn find(string: &[u8], pat: &[u8]) -> Option<usize> {
    (0..=string.len().checked_sub(pat.len())?)
        .find(|&x| string[x..].starts_with(pat))
}

#[cfg(not(feature = "memchr"))]
fn rfind(string: &[u8], pat: &[u8]) -> Option<usize> {
    (pat.len()..=string.len())
        .rfind(|&x| string[..x].ends_with(pat))
        .map(|x| x - pat.len())
}

pub(super) unsafe fn os_str(string: &[u8]) -> &OsStr {
    // SAFETY: This function has equivalent safety requirements.
    unsafe { OsStr::from_encoded_bytes_unchecked(string) }
}

fn split_once<'a, 'b, F, P>(
    string: &'a OsStr,
    pat: &'b P,
    find_fn: F,
) -> Option<(&'a OsStr, &'a OsStr)>
where
    F: FnOnce(&OsStr, &'b str) -> Option<usize>,
    P: EncodedPattern,
{
    let pat = pat.__as_str();

    let index = find_fn(string, pat)?;
    let string = string.as_encoded_bytes();
    let prefix = &string[..index];
    let suffix = &string[index + pat.len()..];
    // SAFETY: These substrings were separated by a UTF-8 string.
    Some(unsafe { (os_str(prefix), os_str(suffix)) })
}

fn trim_matches<'a, 'b, P, F>(
    mut string: &'a OsStr,
    pat: &'b P,
    strip_fn: F,
) -> &'a OsStr
where
    F: for<'c> Fn(&'c OsStr, &'b str) -> Option<&'c OsStr>,
    P: EncodedPattern,
{
    let pat = pat.__as_str();

    if !pat.is_empty() {
        while let Some(substring) = strip_fn(string, pat) {
            string = substring;
        }
    }
    string
}

fn trim_end_matches<'a, P>(string: &'a OsStr, pat: &P) -> &'a OsStr
where
    P: EncodedPattern,
{
    trim_matches(string, pat, OsStrBytesExt::strip_suffix)
}

fn trim_start_matches<'a, P>(string: &'a OsStr, pat: &P) -> &'a OsStr
where
    P: EncodedPattern,
{
    trim_matches(string, pat, OsStrBytesExt::strip_prefix)
}

#[cfg(not(feature = "conversions"))]
trait OsStrBytes: private::Sealed {}

#[cfg(not(feature = "conversions"))]
impl OsStrBytes for OsStr {}

/// An extension trait providing additional methods to [`OsStr`].
///
/// In most cases, this trait will prevent needing to call
/// [`OsStr::as_encoded_bytes`] and potentially violating invariants of the
/// internal encoding for [`OsStr`].
///
/// # Indices
///
/// Methods of this struct that accept indices require that the index lie on a
/// UTF-8 boundary. Although it is possible to manipulate platform strings
/// based on other indices, this crate currently does not support them for
/// slicing methods. They are not currently possible to support safely and are
/// generally not necessary. However, all indices returned by this trait can be
/// passed to other methods.
///
/// # Complexity
///
/// All searching methods have worst-case multiplicative time complexity (i.e.,
/// `O(self.len() * pat.len())`). Enabling the "memchr" feature allows these
/// methods to instead run in linear time in the worst case (documented for
/// [`memchr::memmem::find`][memchr_complexity]).
///
/// [memchr_complexity]: ::memchr::memmem::find#complexity
#[cfg_attr(not(feature = "conversions"), allow(private_bounds))]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait OsStrBytesExt: OsStrBytes {
    /// Equivalent to [`str::contains`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert!(os_string.contains("oo"));
    /// assert!(!os_string.contains("of"));
    /// ```
    #[must_use]
    fn contains<P>(&self, pat: P) -> bool
    where
        P: Pattern;

    /// Equivalent to [`str::ends_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert!(os_string.ends_with("bar"));
    /// assert!(!os_string.ends_with("foo"));
    /// ```
    #[must_use]
    fn ends_with<P>(&self, pat: P) -> bool
    where
        P: Pattern;

    if_conversions! {
        /// Equivalent to [`str::ends_with`] but accepts this type for the
        /// pattern.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsStr;
        ///
        /// use os_str_bytes::OsStrBytesExt;
        ///
        /// let os_string = OsStr::new("foobar");
        /// assert!(os_string.ends_with_os(OsStr::new("bar")));
        /// assert!(!os_string.ends_with_os(OsStr::new("foo")));
        /// ```
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "conversions"))
        )]
        #[must_use]
        fn ends_with_os(&self, pat: &Self) -> bool;
    }

    /// Equivalent to [`str::find`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!(Some(1), os_string.find("o"));
    /// assert_eq!(None, os_string.find("of"));
    /// ```
    #[must_use]
    fn find<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern;

    /// Equivalent to the [`Index::index`] implementation for [`prim@str`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!("foo", os_string.index(..3));
    /// assert_eq!("bar", os_string.index(3..));
    /// ```
    ///
    /// [`Index::index`]: std::ops::Index::index
    /// [valid boundary]: #indices
    #[must_use]
    #[track_caller]
    fn index<I>(&self, index: I) -> &Self
    where
        I: SliceIndex;

    /// Equivalent to [`str::rfind`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!(Some(2), os_string.rfind("o"));
    /// assert_eq!(None, os_string.rfind("of"));
    /// ```
    #[must_use]
    fn rfind<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern;

    /// Equivalent to [`str::rsplit_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!(
    ///     Some((OsStr::new("fo"), OsStr::new("bar"))),
    ///     os_string.rsplit_once("o"),
    /// );
    /// assert_eq!(None, os_string.rsplit_once("of"));
    /// ```
    #[must_use]
    fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern;

    /// Equivalent to [`str::split`], but empty patterns are not accepted.
    ///
    /// # Panics
    ///
    /// Panics if the pattern is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::new("foobar");
    /// assert!(raw.split("o").eq(["f", "", "bar"]));
    /// ```
    #[track_caller]
    fn split<P>(&self, pat: P) -> Split<'_, P>
    where
        P: Pattern;

    /// Equivalent to [`str::split_at`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!(
    ///     (OsStr::new("fo"), OsStr::new("obar")),
    ///     os_string.split_at(2),
    /// );
    /// ```
    ///
    /// [valid boundary]: #indices
    #[must_use]
    #[track_caller]
    fn split_at(&self, mid: usize) -> (&Self, &Self);

    /// Equivalent to [`str::split_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert_eq!(
    ///     Some((OsStr::new("f"), OsStr::new("obar"))),
    ///     os_string.split_once("o"),
    /// );
    /// assert_eq!(None, os_string.split_once("of"));
    /// ```
    #[must_use]
    fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern;

    /// Equivalent to [`str::starts_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("foobar");
    /// assert!(os_string.starts_with("foo"));
    /// assert!(!os_string.starts_with("bar"));
    /// ```
    #[must_use]
    fn starts_with<P>(&self, pat: P) -> bool
    where
        P: Pattern;

    if_conversions! {
        /// Equivalent to [`str::starts_with`] but accepts this type for the
        /// pattern.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsStr;
        ///
        /// use os_str_bytes::OsStrBytesExt;
        ///
        /// let os_string = OsStr::new("foobar");
        /// assert!(os_string.starts_with_os(OsStr::new("foo")));
        /// assert!(!os_string.starts_with_os(OsStr::new("bar")));
        /// ```
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "conversions"))
        )]
        #[must_use]
        fn starts_with_os(&self, pat: &Self) -> bool;
    }

    /// Equivalent to [`str::strip_prefix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("111foo1bar111");
    /// assert_eq!(
    ///     Some(OsStr::new("11foo1bar111")),
    ///     os_string.strip_prefix("1"),
    /// );
    /// assert_eq!(None, os_string.strip_prefix("o"));
    /// ```
    #[must_use]
    fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern;

    /// Equivalent to [`str::strip_suffix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("111foo1bar111");
    /// assert_eq!(
    ///     Some(OsStr::new("111foo1bar11")),
    ///     os_string.strip_suffix("1"),
    /// );
    /// assert_eq!(None, os_string.strip_suffix("o"));
    /// ```
    #[must_use]
    fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern;

    /// Equivalent to [`str::trim_end_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("111foo1bar111");
    /// assert_eq!("111foo1bar", os_string.trim_end_matches("1"));
    /// assert_eq!("111foo1bar111", os_string.trim_end_matches("o"));
    /// ```
    #[must_use]
    fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern;

    /// Equivalent to [`str::trim_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("111foo1bar111");
    /// assert_eq!("foo1bar", os_string.trim_matches("1"));
    /// assert_eq!("111foo1bar111", os_string.trim_matches("o"));
    /// ```
    #[must_use]
    fn trim_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern;

    /// Equivalent to [`str::trim_start_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    ///
    /// use os_str_bytes::OsStrBytesExt;
    ///
    /// let os_string = OsStr::new("111foo1bar111");
    /// assert_eq!("foo1bar111", os_string.trim_start_matches("1"));
    /// assert_eq!("111foo1bar111", os_string.trim_start_matches("o"));
    /// ```
    #[must_use]
    fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern;
}

impl OsStrBytesExt for OsStr {
    #[inline]
    fn contains<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.find(pat).is_some()
    }

    #[inline]
    fn ends_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        self.as_encoded_bytes().ends_with(pat)
    }

    if_conversions! {
        #[inline]
        fn ends_with_os(&self, pat: &Self) -> bool {
            raw::ends_with(&self.to_raw_bytes(), &pat.to_raw_bytes())
        }
    }

    #[inline]
    fn find<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        find(self.as_encoded_bytes(), pat)
    }

    #[inline]
    fn index<I>(&self, index: I) -> &Self
    where
        I: SliceIndex,
    {
        index.index(self)
    }

    #[inline]
    fn rfind<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        rfind(self.as_encoded_bytes(), pat)
    }

    #[inline]
    fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        split_once(self, &pat.__encode(), Self::rfind)
    }

    #[inline]
    fn split<P>(&self, pat: P) -> Split<'_, P>
    where
        P: Pattern,
    {
        Split::new(self, pat)
    }

    #[inline]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        check_bound(self, mid);

        let (prefix, suffix) = self.as_encoded_bytes().split_at(mid);
        // SAFETY: These substrings were separated by a valid boundary.
        unsafe { (os_str(prefix), os_str(suffix)) }
    }

    #[inline]
    fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        split_once(self, &pat.__encode(), Self::find)
    }

    #[inline]
    fn starts_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        self.as_encoded_bytes().starts_with(pat)
    }

    if_conversions! {
        #[inline]
        fn starts_with_os(&self, pat: &Self) -> bool {
            raw::starts_with(&self.to_raw_bytes(), &pat.to_raw_bytes())
        }
    }

    #[inline]
    fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        // SAFETY: This substring was separated by a UTF-8 string.
        self.as_encoded_bytes()
            .strip_prefix(pat)
            .map(|x| unsafe { os_str(x) })
    }

    #[inline]
    fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__as_bytes();

        // SAFETY: This substring was separated by a UTF-8 string.
        self.as_encoded_bytes()
            .strip_suffix(pat)
            .map(|x| unsafe { os_str(x) })
    }

    #[inline]
    fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        trim_end_matches(self, &pat.__encode())
    }

    #[inline]
    fn trim_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        trim_end_matches(trim_start_matches(self, &pat), &pat)
    }

    #[inline]
    fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        trim_start_matches(self, &pat.__encode())
    }
}

pub trait SliceIndex {
    fn index(self, string: &OsStr) -> &OsStr;
}

macro_rules! r#impl {
    ( $type:ty $(, $var:ident , $($bound:expr),+)? ) => {
        impl SliceIndex for $type {
            #[inline]
            fn index(self, string: &OsStr) -> &OsStr {
                $(
                    let $var = &self;
                    $(check_bound(string, $bound);)+
                )?

                // SAFETY: This substring is separated by valid boundaries.
                unsafe { os_str(&string.as_encoded_bytes()[self]) }
            }
        }
    };
}
r#impl!(Range<usize>, x, x.start, x.end);
r#impl!(RangeFrom<usize>, x, x.start);
r#impl!(RangeFull);
// [usize::MAX] will always be a valid inclusive end index.
#[rustfmt::skip]
r#impl!(RangeInclusive<usize>, x, *x.start(), x.end().wrapping_add(1));
r#impl!(RangeTo<usize>, x, x.end);
r#impl!(RangeToInclusive<usize>, x, x.end.wrapping_add(1));
