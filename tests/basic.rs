use std::fmt::Debug;

use maybe_debug::{MaybeDebug, maybe_debug};

#[rustversion::nightly]
const NIGHTLY: bool = true;
#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

fn expected_slice_fallback<T>(val: &[T]) -> String {
    format!("{:?}", MaybeDebug::fallback_slice(val))
}
fn expected_fallback<T: ?Sized>() -> String {
    format!("{:?}", MaybeDebug::fallback::<T>())
}

fn assert_debug<T: Debug>(val: T, expected: impl Into<String>) {
    assert_eq!(format!("{:?}", val), expected.into());
} 
fn assert_debug_passthrough<T: ?Sized + Debug>(val: &T) {
    assert_debug(maybe_debug(val), if NIGHTLY {
        format!("{:?}", val)
    } else {
        expected_fallback::<T>()
    });
} 

#[derive(Copy, Clone)]
struct NotDebug(u32);

#[test]
fn debug_regular() {
    #[derive(Debug)]
    struct Foo(u32, String);
    assert_debug_passthrough(
        &Foo(5, "too".into())
    );
    assert_debug_passthrough(&String::from("too"));
    assert_debug(
        maybe_debug(&NotDebug(15)),
        expected_fallback::<NotDebug>()
    );
}
#[test]
fn debug_slices() {
    let v = vec![1, 4, 8];
    assert_debug_passthrough(v.as_slice());
    assert_debug_passthrough(b"you are a superstar");
    let non_debug = vec![NotDebug(18); 4];
    assert_debug(
        maybe_debug(non_debug.as_slice()),
        if NIGHTLY {
            expected_slice_fallback(non_debug.as_slice())
        } else {
            expected_fallback::<[NotDebug]>()
        }
    );
}
#[test]
fn debug_str() {
    assert_debug_passthrough("foo");
}

#[test]
fn cast_debug() {
    let s = String::from("foo baz");
    assert_eq!(
        maybe_debug::cast_debug(&s).map(|r| r as *const _ as *const String),
        if NIGHTLY {
            Some(&s as &dyn Debug as *const _ as *const String)
        } else {
            None
        }
    );
}