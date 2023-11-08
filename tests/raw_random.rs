#![cfg(feature = "raw_os_str")]

#[macro_use]
mod common;

if_conversions! {
    use std::ffi::OsStr;

    use os_str_bytes::OsStrBytesExt;

    mod random_common;
    use random_common::ITERATIONS;
    use random_common::SMALL_LENGTH;
}

if_conversions! {
    #[test]
    fn test_complex() {
        macro_rules! test {
            ( $result:expr , $method:ident ( $(& $arg:ident),+) ) => {
                assert_eq!(
                    $result,
                    OsStr::$method($(&$arg),+),
                    concat!(stringify!($method), "({:?}, {:?})"),
                    $($arg,)+
                );
            };
        }

        for _ in 0..ITERATIONS {
            let mut string = random_common::fastrand_os_string(SMALL_LENGTH);
            let prefix = string.clone();
            let suffix = random_common::fastrand_os_string(SMALL_LENGTH);
            string.push(&suffix);

            test!(true, ends_with_os(&string, &suffix));
            test!(true, starts_with_os(&string, &prefix));

            if prefix != suffix {
                test!(false, ends_with_os(&string, &prefix));
                test!(false, starts_with_os(&string, &suffix));
            }
        }
    }
}
