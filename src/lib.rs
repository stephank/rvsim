#![warn(missing_docs)]
#![allow(clippy::doc_markdown)]

//! A RISC-V simulator implementing RV32G[C].
//!
//! ## Usage
//!
//! The primary workhorse in this crate is the `Interp`. It takes a `CpuState`, `Memory` and
//! `Clock`, then simulates a virtual CPU using these resources. `CpuState` is a struct, while
//! `Memory` and `Clock` are traits, allowing complete control over the structure of the rest of
//! the virtual machine.
//!
//! When using the crate feature `serialize`, a `CpuState` can be serialized (and deserialized) in
//! order to suspend a virtual machine to persistent storage.
//!
//! A very basic ELF parser is also provided in the `elf` module. Rvsim itself uses this parser to
//! run the official RISC-V test suite.
//!
//! ## Example
//!
//! ```
//! extern crate rvsim;
//!
//! use std::io::Write;
//!
//! /// A simple `Memory` implementation, that creates an address space with just some DRAM.
//! struct SimpleMemory {
//!     dram: Vec<u8>,
//! }
//!
//! impl SimpleMemory {
//!     const DRAM_BASE: u32 = 0x1000_0000;
//!     const DRAM_SIZE: usize = 0x10_0000;
//!
//!     fn new() -> Self {
//!         Self { dram: vec![0; Self::DRAM_SIZE] }
//!     }
//! }
//!
//! /// Our implementation of `Memory` builds a simple memory map.
//! ///
//! /// The `Memory` trait is also implemented for `[u8]`, so we can simply delegate to it, after
//! /// translating the address.
//! ///
//! /// The condition here only checks the start address of DRAM, because the upper bound is
//! /// already checked by the `[u8]` implementation. This type of memory map can be easily
//! /// extended by adding more `else if` clauses, working through blocks of memory from highest
//! /// base address to lowest.
//! impl rvsim::Memory for SimpleMemory {
//!     fn access<T: Copy>(&mut self, addr: u32, access: rvsim::MemoryAccess<T>) -> bool {
//!         if addr >= Self::DRAM_BASE {
//!             rvsim::Memory::access(&mut self.dram[..], addr - Self::DRAM_BASE, access)
//!         } else {
//!             false
//!         }
//!     }
//! }
//!
//! fn main() {
//!     // Create the `SimpleMemory` and load some code into it.
//!     // Writing to the start of DRAM will put the code at `DRAM_BASE` in the address space.
//!     let mut mem = SimpleMemory::new();
//!     (&mut mem.dram[..]).write_all(&[
//!         0x73, 0x00, 0x10, 0x00 // `EBREAK`
//!     ]).unwrap();
//!
//!     // We can use the very basic `Clock` implementation that is provided.
//!     let mut clock = rvsim::SimpleClock::new();
//!
//!     // Create the virtual CPU state, setting the PC to the start of our program.
//!     let mut state = rvsim::CpuState::new(SimpleMemory::DRAM_BASE);
//!
//!     // Run until the program stops.
//!     let mut interp = rvsim::Interp::new(&mut state, &mut mem, &mut clock);
//!     let (err, op) = interp.run();
//!
//!     // The program should've stopped at the `EBREAK` instruction.
//!     assert_eq!(err, rvsim::CpuError::Ebreak);
//!     assert_eq!(op, Some(rvsim::Op::Ebreak));
//! }
//! ```
//!
//! ## Current limitations
//!
//!  - Supports only little-endian hosts.
//!  - Windows support needs work.
//!
//! ## License
//!
//! Rvsim uses the MIT license, but includes portions of Berkeley SoftFloat, which
//! uses the BSD 3-clause license. For details, see the [COPYING.md] file.
//!
//!  [COPYING.md]: https://github.com/stephank/rvsim/blob/main/COPYING.md

#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde_derive;

#[allow(unused_parens)]
mod cpu;

pub mod elf;
#[cfg(feature = "rv32fd")]
pub mod softfloat;

pub use cpu::*;
