include-merkle
========

[![include-merkle on travis-ci.com](https://travis-ci.com/gwihlidal/include-merkle-rs.svg?branch=master)](https://travis-ci.com/gwihlidal/include-merkle-rs)
[![Latest version](https://img.shields.io/crates/v/include-merkle.svg)](https://crates.io/crates/include-merkle)
[![Documentation](https://docs.rs/include-merkle/badge.svg)](https://docs.rs/include-merkle)
[![](https://tokei.rs/b1/github/gwihlidal/include-merkle-rs)](https://github.com/gwihlidal/include-merkle-rs)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![APACHE2](https://img.shields.io/badge/license-APACHE2-blue.svg)

Functionality for generating a Merkle-tree of a given text file with include references, replacing includes paths with a deterministic versioned identity, and also functionality for flattening include directives into a single file. The primary motivation is compiling shaders for various graphics APIs, but the the functionality can apply to a variety of source code parsing use cases.

- [Documentation](https://docs.rs/include-merkle)
- [Release Notes](https://github.com/gwihlidal/include-merkle-rs/releases)

## Example

```
cargo run --example main
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
include-merkle = "0.1.0"
```

and add this to your crate root:

```rust
extern crate include_merkle;
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/gwihlidal/include-merkle-rs/issues) to see what
known improvements are documented.

## Code of Conduct

Contribution to the include-merkle crate is organized under the terms of the
Contributor Covenant, the maintainer of include-merkle, @gwihlidal, promises to
intervene to uphold that code of conduct.