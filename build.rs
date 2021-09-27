/*
 * NOTE: This is necessary because inner macro attributes are unstable
 * 
 * In other words, we can't do 
 * #![rustversion::attr(nightly, feature(...))]
 * on stable, which kind of defeats the purpose...
 */
#[rustversion::nightly]
const NIGHTLY: bool = true;

#[rustversion::not(nightly)]
const NIGHTLY: bool = false;

fn main() {
    if NIGHTLY {
        println!(r#"cargo:rustc-cfg=maybe_debug_nightly"#);
    }
}