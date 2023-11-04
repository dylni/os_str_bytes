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
        ( $name:ident , $index:literal ) => {
            // https://github.com/rust-lang/rust/issues/88430
            #[test]
            fn $name() {
                let error =
                    panic::catch_unwind(|| RAW_WTF8_STRING.index($index..))
                        .expect_err("test did not panic as expected");
                let error: &String =
                    error.downcast_ref().expect("incorrect panic message type");
                assert_eq!(
                    &format!("byte index {} is not a valid boundary", $index),
                    error,
                );
            }
        };
    }
    test!(test_4, 4);
    test!(test_5, 5);
    test!(test_7, 7);
    test!(test_8, 8);
    test!(test_9, 9);
}

#[test]
fn test_panics() {
    #[track_caller]
    fn test<F, R>(f: F)
    where
        F: FnOnce() -> R + UnwindSafe,
    {
        assert!(panic::catch_unwind(f).is_err());
    }

    let string = RawOsStr::new("\u{F6}");
    test(|| string.index(1..2));
    test(|| string.index(0..1));
    test(|| string.index(1..));
    test(|| string.index(0..=0));
    test(|| string.index(..1));
    test(|| string.index(..=0));
    test(|| string.split_at(1));
}
