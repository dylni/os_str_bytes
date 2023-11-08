//! Iterators provided by this crate.

#![cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]

use std::ffi::OsStr;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::iter::FusedIterator;
use std::mem;
use std::str;

use super::ext;
use super::pattern::Encoded;
use super::NonUnicodeOsStr;
use super::OsStrBytesExt;
use super::Pattern;
use super::RawOsStr;

// [memchr::memmem::FindIter] is not currently used, since this struct would
// become self-referential. Additionally, that iterator does not implement
// [DoubleEndedIterator], and its implementation would likely require
// significant changes to implement that trait.
/// The iterator returned by [`OsStrBytesExt::split`].
#[must_use]
pub struct Split<'a, P>
where
    P: Pattern,
{
    string: Option<&'a OsStr>,
    pat: P::__Encoded,
}

impl<'a, P> Split<'a, P>
where
    P: Pattern,
{
    #[track_caller]
    pub(super) fn new(string: &'a OsStr, pat: P) -> Self {
        let pat = pat.__encode();
        assert!(
            !pat.__as_str().is_empty(),
            "cannot split using an empty pattern",
        );
        Self {
            string: Some(string),
            pat,
        }
    }
}

macro_rules! impl_next {
    ( $self:ident , $split_method:ident , $swap:expr ) => {{
        $self
            .string?
            .$split_method($self.pat.__as_str())
            .map(|(mut substring, mut string)| {
                if $swap {
                    mem::swap(&mut substring, &mut string);
                }
                $self.string = Some(string);
                substring
            })
            .or_else(|| $self.string.take())
    }};
}

impl<P> Clone for Split<'_, P>
where
    P: Pattern,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            string: self.string,
            pat: self.pat.clone(),
        }
    }
}

impl<P> Debug for Split<'_, P>
where
    P: Pattern,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Split")
            .field("string", &self.string)
            .field("pat", &self.pat)
            .finish()
    }
}

impl<P> DoubleEndedIterator for Split<'_, P>
where
    P: Pattern,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        impl_next!(self, rsplit_once, true)
    }
}

impl<P> FusedIterator for Split<'_, P> where P: Pattern {}

impl<'a, P> Iterator for Split<'a, P>
where
    P: Pattern,
{
    type Item = &'a OsStr;

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn next(&mut self) -> Option<Self::Item> {
        impl_next!(self, split_once, false)
    }
}

/// The iterator returned by [`RawOsStr::split`].
#[must_use]
pub struct RawSplit<'a, P>(Split<'a, P>)
where
    P: Pattern;

impl<'a, P> RawSplit<'a, P>
where
    P: Pattern,
{
    #[track_caller]
    pub(super) fn new(string: &'a RawOsStr, pat: P) -> Self {
        Self(Split::new(string.as_os_str(), pat))
    }
}

impl<P> Clone for RawSplit<'_, P>
where
    P: Pattern,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<P> Debug for RawSplit<'_, P>
where
    P: Pattern,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RawSplit").field(&self.0).finish()
    }
}

impl<P> DoubleEndedIterator for RawSplit<'_, P>
where
    P: Pattern,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(RawOsStr::new)
    }
}

impl<P> FusedIterator for RawSplit<'_, P> where P: Pattern {}

impl<'a, P> Iterator for RawSplit<'a, P>
where
    P: Pattern,
{
    type Item = &'a RawOsStr;

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last().map(RawOsStr::new)
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(RawOsStr::new)
    }
}

/// The iterator returned by [`OsStrBytesExt::utf8_chunks`].
///
/// [`OsStrBytesExt::utf8_chunks`]: super::OsStrBytesExt::utf8_chunks
#[derive(Clone, Debug)]
#[must_use]
pub struct Utf8Chunks<'a> {
    string: &'a OsStr,
    invalid_length: usize,
}

impl<'a> Utf8Chunks<'a> {
    pub(super) fn new(string: &'a OsStr) -> Self {
        Self {
            string,
            invalid_length: 0,
        }
    }
}

impl FusedIterator for Utf8Chunks<'_> {}

impl<'a> Iterator for Utf8Chunks<'a> {
    type Item = (&'a NonUnicodeOsStr, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let string = self.string.as_encoded_bytes();
        if string.is_empty() {
            debug_assert_eq!(0, self.invalid_length);
            return None;
        }

        loop {
            let (invalid, substring) = string.split_at(self.invalid_length);

            let valid = match str::from_utf8(substring) {
                Ok(valid) => {
                    self.string = OsStr::new("");
                    self.invalid_length = 0;
                    valid
                }
                Err(error) => {
                    let (valid, substring) =
                        substring.split_at(error.valid_up_to());

                    let invalid_length =
                        error.error_len().unwrap_or_else(|| substring.len());
                    if valid.is_empty() {
                        self.invalid_length += invalid_length;
                        continue;
                    }
                    // SAFETY: This substring was separated by a UTF-8 string.
                    self.string = unsafe { ext::os_str(substring) };
                    self.invalid_length = invalid_length;

                    // SAFETY: This slice was validated to be UTF-8.
                    unsafe { str::from_utf8_unchecked(valid) }
                }
            };

            // SAFETY: This substring was separated by a UTF-8 string and
            // validated to not be UTF-8.
            let invalid = unsafe { NonUnicodeOsStr::new_unchecked(invalid) };
            return Some((invalid, valid));
        }
    }
}
