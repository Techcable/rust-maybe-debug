maybe-debug
===========
Implement `Debug` for anything via specialization.

Lets say you have the following function and you want
to insert a `dbg!()` statement inside the loop.
```compile_fail
fn sort<T>(target: &mut [T]) {
    for (i, val) in target.iter().enumerate() {
        dbg!(i);
        // various sorting goodness
        dbg!(i, val); // ERROR: T is not Debug
    }
}
```

You can use `maybe_debug::maybe_debug()` to work around this.
If `T` is `Debug` it will 'cast' it. If `T` is !Debug, it will
fallback to a reasonable default (printing the type name).

```
fn sort<T>(target: &mut [T]) {
    for (i, val) in target.iter().enumerate() {
        maybe_debug::dbg!(i);
        // various sorting goodness
        maybe_debug::dbg!(i, val); // On nightly, will specialize if 'T: Debug'
    }
}
```

This has a fallback to work on stable Rust (without specialization).
In that case, the "cast" always fails and `maybe_debug` will unconditionally
use the fallback.

