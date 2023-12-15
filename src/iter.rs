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

macro_rules! r#impl {
    (
        $(#[ $attr:meta ])* $name:ident ,
        $(#[ $raw_attr:meta ])* $raw_name:ident ,
        $split_method:ident ,
        $reverse:expr ,
    ) => {
        // [memchr::memmem::FindIter] would make this struct self-referential.
        #[must_use]
        $(#[$attr])*
        pub struct $name<'a, P>
        where
            P: Pattern,
        {
            string: Option<&'a OsStr>,
            pat: P::__Encoded,
        }

        impl<'a, P> $name<'a, P>
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

        impl<P> Clone for $name<'_, P>
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

        impl<P> Debug for $name<'_, P>
        where
            P: Pattern,
        {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("string", &self.string)
                    .field("pat", &self.pat)
                    .finish()
            }
        }

        impl<P> FusedIterator for $name<'_, P> where P: Pattern {}

        impl<'a, P> Iterator for $name<'a, P>
        where
            P: Pattern,
        {
            type Item = &'a OsStr;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.string?
                    .$split_method(self.pat.__as_str())
                    .map(|(mut substring, mut string)| {
                        if $reverse {
                            mem::swap(&mut substring, &mut string);
                        }
                        self.string = Some(string);
                        substring
                    })
                    .or_else(|| self.string.take())
            }
        }

        #[must_use]
        $(#[$raw_attr])*
        pub struct $raw_name<'a, P>($name<'a, P>)
        where
            P: Pattern;

        impl<'a, P> $raw_name<'a, P>
        where
            P: Pattern,
        {
            #[track_caller]
            pub(super) fn new(string: &'a RawOsStr, pat: P) -> Self {
                Self($name::new(string.as_os_str(), pat))
            }
        }

        impl<P> Clone for $raw_name<'_, P>
        where
            P: Pattern,
        {
            #[inline]
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl<P> Debug for $raw_name<'_, P>
        where
            P: Pattern,
        {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($raw_name)).field(&self.0).finish()
            }
        }

        impl<P> FusedIterator for $raw_name<'_, P> where P: Pattern {}

        impl<'a, P> Iterator for $raw_name<'a, P>
        where
            P: Pattern,
        {
            type Item = &'a RawOsStr;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(RawOsStr::new)
            }
        }
    };
}
r#impl!(
    /// The iterator returned by [`OsStrBytesExt::split`].
    Split,
    /// The iterator returned by [`RawOsStr::split`].
    RawSplit,
    split_once,
    false,
);
r#impl!(
    /// The iterator returned by [`OsStrBytesExt::rsplit`].
    RSplit,
    /// The iterator returned by [`RawOsStr::rsplit`].
    RawRSplit,
    rsplit_once,
    true,
);

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
