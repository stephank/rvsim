# rvsim

[![Docs](https://docs.rs/rvsim/badge.svg)](https://docs.rs/rvsim)
[![Crate](https://img.shields.io/crates/v/rvsim.svg)](https://crates.io/crates/rvsim)

A RISC-V simulator implementing RV32G[C], written in Rust.

See the [documentation] for usage.

[documentation]: https://docs.rs/rvsim

## Current limitations

- Supports only little-endian hosts.
- Windows support needs work.

## Features

- `rv32c` enable RV32C compressed instruction set support
- `rv32fd` enables RV32F (Single-Precision Floating-Point) and RV32F (Double-Precision Floating-Point) instruction set support (default)
- `serde` enable serialization support

## License

Rvsim uses the MIT license, but includes portions of Berkeley SoftFloat, used
when the 'rv32fd' feature is enabled (default). Berkely SoftFloat uses the BSD
3-clause license. For details, see the [COPYING.md](./COPYING.md) file.
