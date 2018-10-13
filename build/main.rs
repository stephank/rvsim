extern crate cc;
extern crate regex;

mod cpu;
#[cfg(feature = "rv32fd")]
mod softfloat;

fn main() {
    cpu::build();
    #[cfg(feature = "rv32fd")]
    softfloat::build();
}
