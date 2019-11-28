# OsStr Bytes

Traits for converting between byte sequences and platform-native strings.

This crate allows interacting with the bytes stored internally by [`OsStr`] and
[`OsString`], without resorting to panics or data corruption for invalid UTF-8.
Thus, methods can be used that are already defined on [`[u8]`][slice] and
[`Vec<u8>`].

Typically, the only way to losslessly construct [`OsStr`] or [`OsString`] from
a byte sequence is to use `OsString::from(String::from(bytes).unwrap())`, which
requires the bytes to be valid in UTF-8. However, since this crate makes
conversions directly between the platform encoding and raw bytes, even some
strings invalid in UTF-8 can be converted.

[![GitHub Build Status](https://github.com/dylni/os_str_bytes/workflows/build/badge.svg?branch=master)](https://github.com/dylni/os_str_bytes/actions)

## Usage

Add the following lines to your "Cargo.toml" file:

```toml
[dependencies]
os_str_bytes = "0.1"
```

See the [documentation] for available functionality and examples.

## Rust version support

The minimum supported Rust toolchain version is currently Rust 1.32.0.

[documentation]: https://docs.rs/os_str_bytes
[slice]: https://doc.rust-lang.org/std/primitive.slice.html
[`OsStr`]: https://doc.rust-lang.org/std/ffi/struct.OsStr.html
[`OsString`]: https://doc.rust-lang.org/std/ffi/struct.OsString.html
[`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
