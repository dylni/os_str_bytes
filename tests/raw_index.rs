#![cfg(feature = "raw_os_str")]

use std::ops::Index;
use std::panic;
use std::panic::UnwindSafe;

use os_str_bytes::RawOsStr;

#[macro_use]
mod raw_common;

if_conversions! {
    use raw_common::RAW_WTF8_STRING;
}

const EXPECT_PANIC: bool = !cfg!(unix) || cfg!(feature = "nightly");

if_conversions! {
    #[test]
    fn test_valid() {
        #[track_caller]
        fn test(index: usize) {
            let _ = RAW_WTF8_STRING.index(index..);
        }

        test(0);
        test(1);
        test(2);
        test(3);
        test(6);
        test(10);
        test(11);
        test(12);
        test(13);
    }

    macro_rules! test {
        ( $name:ident , $index:literal , $code_point:expr ) => {
            // https://github.com/rust-lang/rust/issues/88430
            #[test]
            fn $name() {
                let index_fn = || RAW_WTF8_STRING.index($index..);
                if !EXPECT_PANIC {
                    let _ = index_fn();
                    return;
                }

                let error = panic::catch_unwind(index_fn)
                    .expect_err("test did not panic as expected");
                let error: &String =
                    error.downcast_ref().expect("incorrect panic message type");
                let suffix = if cfg!(feature = "nightly") {
                    ""
                } else {
                    concat!("; it is inside ", $code_point)
                };
                assert_eq!(
                    &format!(
                        "byte index {} is not a valid boundary{}",
                        $index, suffix,
                    ),
                    error,
                );
            }
        };
    }
    test!(test_4, 4, "U+D83D (bytes 3..6)");
    test!(test_5, 5, "U+D83D (bytes 3..6)");
    test!(test_7, 7, "U+1F4A9 (bytes 6..10)");
    test!(test_8, 8, "U+1F4A9 (bytes 6..10)");
    test!(test_9, 9, "U+1F4A9 (bytes 6..10)");
}

#[test]
fn test_panics() {
    #[track_caller]
    fn test<F, R>(f: F)
    where
        F: FnOnce() -> R + UnwindSafe,
    {
        assert_eq!(EXPECT_PANIC, panic::catch_unwind(f).is_err());
    }

    let string = RawOsStr::from_str("\u{F6}");
    test(|| string.index(1..2));
    test(|| string.index(0..1));
    test(|| string.index(1..));
    test(|| string.index(0..=0));
    test(|| string.index(..1));
    test(|| string.index(..=0));
    test(|| string.split_at(1));
}
