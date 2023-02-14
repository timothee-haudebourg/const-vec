# Const Vec

[![CI](https://github.com/timothee-haudebourg/const-vec/workflows/Continuous%20Integration/badge.svg)](https://github.com/timothee-haudebourg/const-vec/actions)
[![Crate informations](https://img.shields.io/crates/v/const-vec.svg?style=flat-square)](https://crates.io/crates/const-vec)
[![License](https://img.shields.io/crates/l/const-vec.svg?style=flat-square)](https://github.com/timothee-haudebourg/const-vec#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/const-vec)

<!-- cargo-rdme start -->

This library provides a `Vec`-like data structure called `ConstVec` where
elements can be pushed to the array in an immutable way as long as the
capacity of the vector is large enough).

## Example

```rust
use const_vec::ConstVec;

// Create a new empty `ConstVec` with a capacity of 10 items.
// Note that it is NOT mutable.
let vec = ConstVec::new(10);

// Add a new element in `vec`, without mutating it.
vec.push(42);
```

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
