# QRForge third-party notices

QRForge is distributed under `MIT OR Apache-2.0`. Its locked Rust and npm
dependency graphs contain additional open-source software. The release gate
checks Rust license expressions with `cargo deny` and requires npm lockfile
license metadata for every installed package.

## ZXing-C++ Rust wrapper and native decoder

- Rust package: `zxing-cpp` 0.5.2
- crates.io checksum:
  `d412e2db33c4afe7aac2e90c829938e8dac4dba2e9572d856b3d8eefc702eae9`
- source repository: <https://github.com/zxing-cpp/zxing-cpp/>
- bundled ZXing-C++ version reported by its CMake project: 3.1.0
- license: Apache License 2.0
- representative copyright: Copyright 2019 Axel Waggershauser

QRForge enables the wrapper's `bundled` feature and pins the wrapper to exactly
0.5.2. The Apache License 2.0 text is included in this repository as
`LICENSE-APACHE`.

## Bundled libzint and libzueci sources

The `zxing-cpp` 0.5.2 source archive includes libzint 2.16.0 and libzueci
sources. Although QRForge uses decoder APIs, the wrapper's native bundled build
includes these source trees.

- libzint: Copyright 2009-2025 Robin Stuart and contributors
- libzueci: Copyright 2022 gitlost
- license: BSD 3-Clause

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the copyright notice, conditions,
and disclaimer are retained; project and contributor names may not be used to
endorse derived products without permission. The software is provided “as is,”
without warranty, and its authors are not liable for damages arising from its
use.

## Complete dependency review

This file calls out the native decoder because it is compiled into QRForge.
The authoritative complete component set is `Cargo.lock` plus
`apps/desktop/package-lock.json`. Review and release commands are documented in
`docs/dependency-security.md`.
