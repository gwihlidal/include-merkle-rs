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
$ cargo run --example main
     Running `target\debug\examples\main.exe`
---
Flatten Test
*
Wrote data out to "./output\\EUv1Sh6w3Sie2iSHM3FtLcn9ZeDp3tnGjfKQiVnJVpJB"
---
Patching Test
*
Wrote data out to "./output\\Bqp6piigLu4FCeVrbXxdifHFuZvAJdWHydHKUbNDpqdt"
Wrote data out to "./output\\E3fsGCD89GRn5xJssirwFKiSQE58qgTPbC4wfA5QPSbz"
Wrote data out to "./output\\B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL"
Wrote data out to "./output\\J2JZmZHPNWyxWCg4rnYPxY88PpbQUt4U1tjBzus6RrSo"
Wrote data out to "./output\\EWqGhBhDUMQk3ggrxyEWDTFPSzsWfFgz9J1V68jjZRNd"
Wrote data out to "./output\\8d2bKyA5JLhnJNUyfg4NNbpHCXHZfppLWUcvFWoGE1QH"
Wrote data out to "./output\\8Jmp4fnvscv4XNeoWBf4adhpaWGGZLZwwuNqN5UB6tRA"
Wrote data out to "./output\\2gMYKEwiVv3xLz2eqVD7r3vASvxEBB2SCR4ZMQJTb12h"
digraph {
    0 [label="(s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:[\"ColorSpaceUtility.hlsli\"])"]
    1 [label="(s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:[\"ColorSpaceUtility.hlsli\"])"]
    2 [label="(s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:[\"ColorSpaceUtility.hlsli\"])"]
    3 [label="(s:[4Ei7qCx94ruYT6bUmKXkj1QCGXKu4raGTQTSSbe6d4UN] p:[Bqp6piigLu4FCeVrbXxdifHFuZvAJdWHydHKUbNDpqdt] f:[\"PixelPacking_RGBE.hlsli\"])"]
    4 [label="(s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:[\"ColorSpaceUtility.hlsli\"])"]
    5 [label="(s:[HHjaptvgha9jdPRkyZE3gFURwU5B4KbeT9BRtZAhneXX] p:[E3fsGCD89GRn5xJssirwFKiSQE58qgTPbC4wfA5QPSbz] f:[\"PixelPacking_RGBM.hlsli\"])"]
    6 [label="(s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:[\"ColorSpaceUtility.hlsli\"])"]
    7 [label="(s:[EuPrBa4zhNBckh4vJ8uWkaULYhFSaFeX2bfKjHpMnQAd] p:[J2JZmZHPNWyxWCg4rnYPxY88PpbQUt4U1tjBzus6RrSo] f:[\"PixelPacking_R11G11B10.hlsli\"])"]
    8 [label="(s:[F82JNfL2QuXCRAYiLcLuRwLQkFjdr6y82akthq7mcSo2] p:[EWqGhBhDUMQk3ggrxyEWDTFPSzsWfFgz9J1V68jjZRNd] f:[\"PixelPacking.hlsli\"])"]
    9 [label="(s:[4Vz59nC3dMpkQPG8Ro55VLjuswZDtmcNnj1tznu2zEoq] p:[8d2bKyA5JLhnJNUyfg4NNbpHCXHZfppLWUcvFWoGE1QH] f:[\"ShaderUtility.hlsli\"])"]
    10 [label="(s:[8Jmp4fnvscv4XNeoWBf4adhpaWGGZLZwwuNqN5UB6tRA] p:[8Jmp4fnvscv4XNeoWBf4adhpaWGGZLZwwuNqN5UB6tRA] f:[\"PostEffectsRS.hlsli\"])"]
    11 [label="(s:[QAFJTtxYCR2WDdx7TCN579qAjsBbK9i4d8XLa3F8U38] p:[2gMYKEwiVv3xLz2eqVD7r3vASvxEBB2SCR4ZMQJTb12h] f:[\"BloomExtractAndDownsampleHdrCS.hlsl\"])"]
    3 -> 2 [label="3"]
    5 -> 4 [label="3"]
    7 -> 6 [label="3"]
    8 -> 1 [label="2"]
    8 -> 3 [label="2"]
    8 -> 5 [label="2"]
    8 -> 7 [label="2"]
    9 -> 0 [label="1"]
    9 -> 8 [label="1"]
    11 -> 9 [label="0"]
    11 -> 10 [label="0"]
}

(s:[QAFJTtxYCR2WDdx7TCN579qAjsBbK9i4d8XLa3F8U38] p:[2gMYKEwiVv3xLz2eqVD7r3vASvxEBB2SCR4ZMQJTb12h] f:["BloomExtractAndDownsampleHdrCS.hlsl"])
├─ (s:[8Jmp4fnvscv4XNeoWBf4adhpaWGGZLZwwuNqN5UB6tRA] p:[8Jmp4fnvscv4XNeoWBf4adhpaWGGZLZwwuNqN5UB6tRA] f:["PostEffectsRS.hlsli"])
└─ (s:[4Vz59nC3dMpkQPG8Ro55VLjuswZDtmcNnj1tznu2zEoq] p:[8d2bKyA5JLhnJNUyfg4NNbpHCXHZfppLWUcvFWoGE1QH] f:["ShaderUtility.hlsli"])
   ├─ (s:[F82JNfL2QuXCRAYiLcLuRwLQkFjdr6y82akthq7mcSo2] p:[EWqGhBhDUMQk3ggrxyEWDTFPSzsWfFgz9J1V68jjZRNd] f:["PixelPacking.hlsli"])
   │  ├─ (s:[EuPrBa4zhNBckh4vJ8uWkaULYhFSaFeX2bfKjHpMnQAd] p:[J2JZmZHPNWyxWCg4rnYPxY88PpbQUt4U1tjBzus6RrSo] f:["PixelPacking_R11G11B10.hlsli"])
   │  │  └─ (s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:["ColorSpaceUtility.hlsli"])
   │  ├─ (s:[HHjaptvgha9jdPRkyZE3gFURwU5B4KbeT9BRtZAhneXX] p:[E3fsGCD89GRn5xJssirwFKiSQE58qgTPbC4wfA5QPSbz] f:["PixelPacking_RGBM.hlsli"])
   │  │  └─ (s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:["ColorSpaceUtility.hlsli"])
   │  ├─ (s:[4Ei7qCx94ruYT6bUmKXkj1QCGXKu4raGTQTSSbe6d4UN] p:[Bqp6piigLu4FCeVrbXxdifHFuZvAJdWHydHKUbNDpqdt] f:["PixelPacking_RGBE.hlsli"])
   │  │  └─ (s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:["ColorSpaceUtility.hlsli"])
   │  └─ (s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:["ColorSpaceUtility.hlsli"])
   └─ (s:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] p:[B6UvYRYE5zP92tmBYKDCAyDSh9b8UuP4NybZCCPeeYcL] f:["ColorSpaceUtility.hlsli"])
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