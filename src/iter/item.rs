use std::ffi::OsStr;
use std::mem;

use crate::ext;

use super::OsUnits;

/// The smallest unit of an unspecified platform-specific encoding permissible
/// for interchange.
///
/// Unlike the results of [`OsStr::as_encoded_bytes`] and similar methods, this
/// unit may be stored and displayed to users in errors.
///
/// Instances are usually constructed using [`OsUnits`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd)]
pub struct OsUnit(pub(crate) u16);

impl OsUnit {
    /// Returns a unique integer unambiguously representing the unit.
    ///
    /// The returned value can be useful for display to users in escaped form
    /// but should otherwise be treated as meaningless.
    ///
    /// # Implementation
    ///
    /// Currently, this method can return:
    /// - bytes (0x00 - 0xFF) -- typically represented as `\xFF`.
    /// - multi-byte units (0x100 - 0x10F_FFF) -- typically represented as
    ///   `\u10FFFF`.
    ///
    /// Future updates may return greater values, so callers are expected to
    /// handle them as well.
    ///
    /// However, the implementation is subject to change. This section is only
    /// informative.
    #[inline]
    #[must_use]
    pub fn to_u64(&self) -> u64 {
        self.0.into()
    }
}

impl From<OsUnit> for u64 {
    #[inline]
    fn from(value: OsUnit) -> Self {
        value.to_u64()
    }
}

/// A container for platform strings containing no unicode characters.
///
/// Instances can only be constructed using [`Utf8Chunks`].
///
/// [`Utf8Chunks`]: super::Utf8Chunks
#[derive(Debug, Hash, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
#[repr(transparent)]
pub struct NonUnicodeOsStr(OsStr);

impl NonUnicodeOsStr {
    unsafe fn from_inner(string: &OsStr) -> &Self {
        // SAFETY: This struct has a layout that makes this operation safe.
        unsafe { mem::transmute(string) }
    }

    pub(super) unsafe fn new_unchecked(string: &[u8]) -> &Self {
        // SAFETY: This method has stricter safety requirements.
        unsafe { Self::from_inner(ext::os_str(string)) }
    }

    /// Converts this representation back to a platform-native string, without
    /// copying or encoding conversion.
    #[inline]
    #[must_use]
    pub fn as_os_str(&self) -> &OsStr {
        &self.0
    }

    /// Converts this representation into units of an unspecified
    /// platform-specific encoding.
    ///
    /// For more information, see [`OsUnit`].
    #[inline]
    pub fn os_units(&self) -> OsUnits<'_> {
        OsUnits::new(self)
    }
}

impl AsRef<OsStr> for NonUnicodeOsStr {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}
