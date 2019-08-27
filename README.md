# Const Vec

<table><tr>
  <td><a href="https://docs.rs/const-vec">Documentation</a></td>
  <td><a href="https://crates.io/crates/const-vec">Crate informations</a></td>
  <td><a href="https://github.com/timothee-haudebourg/const-vec">Repository</a></td>
</tr></table>

Provide a `Vec`-like structure where elements can be pushed to the vector
in an immutable way (as long as the capacity of the vector is large enough).

```rust
extern crate const_vec;

use const_vec::ConstVec;

fn main() {
    // Create a new empty ConstVec of capacity 10.
    // Note that it is NOT mutable.
    let vec = ConstVec::new(10);

    // Add a new element in `vec`, without mutating it.
    vec.push(42);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
