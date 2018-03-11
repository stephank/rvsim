# rvsim

[![Docs](https://docs.rs/rvsim/badge.svg)](https://docs.rs/rvsim)
[![Crate](https://img.shields.io/crates/v/rvsim.svg)](https://crates.io/crates/rvsim)
[![Build Status](https://travis-ci.org/stephank/rvsim.svg?branch=master)](https://travis-ci.org/stephank/rvsim)

A RISC-V simulator implementing RV32G, written in Rust.

See the [documentation] for usage.

 [documentation]: https://docs.rs/rvsim

## Current limitations

 - Supports only little-endian hosts.
 - Windows support needs work.

## License

Rvsim uses the MIT license, but includes portions of Berkeley SoftFloat, which
uses the BSD 3-clause license. For details, see the [COPYING.md](./COPYING.md)
file.
