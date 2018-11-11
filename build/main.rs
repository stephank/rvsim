extern crate cc;
extern crate regex;

mod cpu;
mod softfloat;

fn main() {
    cpu::build();
    softfloat::build();
}
