#![cfg(feature = "raw_os_str")]

use os_str_bytes::RawOsStr;
use os_str_bytes::RawOsString;

mod random_common;
use random_common::ITERATIONS;
use random_common::SMALL_LENGTH;

#[cfg_attr(feature = "nightly", allow(deprecated))]
#[test]
fn test_complex() {
    macro_rules! test {
        ( $result:expr , $method:ident ( $(& $arg:ident),+) ) => {
            assert_eq!(
                $result,
                RawOsStr::$method($(&$arg),+),
                concat!(stringify!($method), "({:?}, {:?})"),
                $($arg,)+
            );
        };
    }

    for _ in 0..ITERATIONS {
        let mut string = random_common::fastrand_os_string(SMALL_LENGTH);
        let prefix = RawOsStr::new(&string).into_owned();
        let suffix = random_common::fastrand_os_string(SMALL_LENGTH);
        string.push(&suffix);

        let string = RawOsString::new(string);
        let suffix = RawOsString::new(suffix);

        test!(true, ends_with_os(&string, &suffix));
        test!(true, starts_with_os(&string, &prefix));

        if prefix != suffix {
            test!(false, ends_with_os(&string, &prefix));
            test!(false, starts_with_os(&string, &suffix));
        }
    }
}
