/// A large enum holding a parsed instruction and its arguments.
#[allow(missing_docs)]
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Op {
    //% variants
}

impl Op {
    /// Parse an instruction. Returns `None` on failure.
    pub fn parse(instr: u32) -> Option<Op> {
        //% parse
    }

    /// Parse a rv32c instruction. Returns `None` on failure.
    #[cfg(feature = "rv32c")]
    pub fn parse_c(instr: u16) -> Option<Op> {
        //% parse_c
    }
}

//
// Matching fields.
//

fn opcode(instr: u32) -> u32 {
    instr & 0b0000_0000_0000_0000_0000_0000_0111_1111
}

#[cfg(feature = "rv32fd")]
fn funct2(instr: u32) -> u32 {
    (instr & 0b0000_0110_0000_0000_0000_0000_0000_0000) >> 25
}

fn funct3(instr: u32) -> u32 {
    (instr & 0b0000_0000_0000_0000_0111_0000_0000_0000) >> 12
}

fn funct5(instr: u32) -> u32 {
    (instr & 0b1111_1000_0000_0000_0000_0000_0000_0000) >> 27
}

fn funct7(instr: u32) -> u32 {
    (instr & 0b1111_1110_0000_0000_0000_0000_0000_0000) >> 25
}

fn funct12(instr: u32) -> u32 { 
    (instr & 0b1111_1111_1111_0000_0000_0000_0000_0000) >> 20
}

fn shtype(instr: u32) -> u32 {
    (instr & 0b1111_1110_0000_0000_0000_0000_0000_0000) >> 25
}

//
// Register fields.
//

fn rd(instr: u32) -> usize {
    ((instr & 0b0000_0000_0000_0000_0000_1111_1000_0000) as usize) >> 7
}

fn rs1(instr: u32) -> usize {
    ((instr & 0b0000_0000_0000_1111_1000_0000_0000_0000) as usize) >> 15
}

fn rs2(instr: u32) -> usize {
    ((instr & 0b0000_0001_1111_0000_0000_0000_0000_0000) as usize) >> 20
}

#[cfg(feature = "rv32fd")]
fn rs3(instr: u32) -> usize {
    ((instr & 0b1111_1000_0000_0000_0000_0000_0000_0000) as usize) >> 27
}

//
// Immediate fields.
//

fn i_imm(instr: u32) -> i32 {
    ((instr & 0b1111_1111_1111_0000_0000_0000_0000_0000) as i32) >> 20
}

fn s_imm(instr: u32) -> i32 {
    (((instr & 0b0000_0000_0000_0000_0000_1111_1000_0000) as i32) >> 7) |
    (((instr & 0b1111_1110_0000_0000_0000_0000_0000_0000) as i32) >> 20)
}

fn b_imm(instr: u32) -> i32 {
    (((instr & 0b0000_0000_0000_0000_0000_1111_0000_0000) as i32) >> 7) |
    (((instr & 0b0111_1110_0000_0000_0000_0000_0000_0000) as i32) >> 20) |
    (((instr & 0b0000_0000_0000_0000_0000_0000_1000_0000) as i32) << 4) |
    (((instr & 0b1000_0000_0000_0000_0000_0000_0000_0000) as i32) >> 19)
}

fn u_imm(instr: u32) -> i32 {
    (instr & 0b1111_1111_1111_1111_1111_0000_0000_0000) as i32
}

fn j_imm(instr: u32) -> i32 {
    (((instr & 0b0111_1111_1110_0000_0000_0000_0000_0000) as i32) >> 20) |
    (((instr & 0b0000_0000_0001_0000_0000_0000_0000_0000) as i32) >> 9) |
    ((instr & 0b0000_0000_0000_1111_1111_0000_0000_0000) as i32) |
    (((instr & 0b1000_0000_0000_0000_0000_0000_0000_0000) as i32) >> 11)
}

//
// Special fields.
//

fn shamt(instr: u32) -> u32 {
    (instr & 0b0000_0001_1111_0000_0000_0000_0000_0000) >> 20
}

fn aq(instr: u32) -> bool {
    (instr & 0b0000_0100_0000_0000_0000_0000_0000_0000) != 0
}

fn rl(instr: u32) -> bool {
    (instr & 0b0000_0010_0000_0000_0000_0000_0000_0000) != 0
}

#[cfg(feature = "rv32fd")]
fn rm(instr: u32) -> u32 {
    (instr & 0b0000_0000_0000_0000_0111_0000_0000_0000) >> 12
}

fn pred(instr: u32) -> u32 {
    (instr & 0b0000_1111_0000_0000_0000_0000_0000_0000) >> 19
}

fn succ(instr: u32) -> u32 {
    (instr & 0b0000_0000_1111_0000_0000_0000_0000_0000) >> 15
}

fn csr(instr: u32) -> u32 {
    (instr & 0b1111_1111_1111_0000_0000_0000_0000_0000) >> 20
}

fn zimm(instr: u32) -> u32 {
    (instr & 0b0000_0000_0000_1111_1000_0000_0000_0000) >> 15
}

fn unused1(instr: u32) -> u32 {
    (instr & 0b1111_0000_0000_0000_0000_0000_0000_0000) >> 23
}

//
// RV32C fields.
//
#[cfg(feature = "rv32c")]
mod rv32c {
    // Opcode selectors.
    pub fn cquad(instr: u16) -> u16 {
        (instr & 0b0000_0000_0000_0011)
    }
    pub fn cfunct3(instr: u16) -> u16 {
        (instr & 0b1110_0000_0000_0000) >> 13
    }
    pub fn cfunct4_l0(instr: u16) -> u16 {
        (instr & 0b0001_0000_0000_0000) >> 12
    }
    pub fn crs1rd_h2(instr: u16) -> u16 {
        (instr & 0b0000_1100_0000_0000) >> 10
    }
    pub fn crs2_h2(instr: u16) -> u16 {
        (instr & 0b0000_0000_0110_0000) >> 5
    }

    // Hardwired register fields.
    pub fn crx0(_instr: u16) -> usize {
        0
    }

    pub fn crra(_instr: u16) -> usize {
        1
    }

    pub fn crsp(_instr: u16) -> usize {
        2
    }

    // Registers x0..x31.
    pub fn crs1rd(instr: u16) -> usize {
        ((instr & 0b0000_1111_1000_0000) as usize) >> 7
    }
    pub fn crs2(instr: u16) -> usize {
        ((instr & 0b0000_0000_0111_1100) as usize) >> 2
    }

    // Registers x8..x15.
    pub fn crs1rdq(instr: u16) -> usize {
        (((instr & 0b0000_0011_1000_0000) as usize) >> 7) + 8
    }
    pub fn crs2q(instr: u16) -> usize {
        (((instr & 0b0000_0000_0001_1100) as usize) >> 2) + 8
    }

    // Hardwired immediates.
    pub fn czero(_instr: u16) -> i32 {
        0
    }

    // Immediates (zero-extended).
    pub fn cimmsh6(instr: u16) -> u32 {
        (((instr & 0b0001_0000_0000_0000) as u32) >> 12) << 5 |
        (((instr & 0b0000_0000_0111_1100) as u32) >> 2)
    }

    pub fn cimmlwsp(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) >> 12) << 5 |
         ((instr & 0b0000_0000_0111_0000) >> 4) << 2 |
         ((instr & 0b0000_0000_0000_1100) >> 2) << 6) as i32
    }

    #[cfg(feature = "rv32fd")]
    pub fn cimmldsp(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) >> 12) << 5 |
         ((instr & 0b0000_0000_0110_0000) >> 5) << 3 |
         ((instr & 0b0000_0000_0001_1100) >> 2) << 6) as i32
    }

    pub fn cimmswsp(instr: u16) -> i32 {
        (((instr & 0b0001_1110_0000_0000) >> 9) << 2 |
         ((instr & 0b0000_0001_1000_0000) >> 7) << 6) as i32
    }

    #[cfg(feature = "rv32fd")]
    pub fn cimmsdsp(instr: u16) -> i32 {
        (((instr & 0b0001_1100_0000_0000) >> 10) << 3 |
         ((instr & 0b0000_0011_1000_0000) >> 7) << 6) as i32
    }

    pub fn cimm4spn(instr: u16) -> i32 {
        (((instr & 0b0001_1000_0000_0000) >> 11) << 4 |
         ((instr & 0b0000_0111_1000_0000) >> 7) << 6 |
         ((instr & 0b0000_0000_0100_0000) >> 6) << 2 |
         ((instr & 0b0000_0000_0010_0000) >> 5) << 3) as i32
    }

    pub fn cimmw(instr: u16) -> i32 {
        (((instr & 0b0001_1100_0000_0000) >> 10) << 3 |
         ((instr & 0b0000_0000_0100_0000) >> 6) << 2 |
         ((instr & 0b0000_0000_0010_0000) >> 5) << 6) as i32
    }

    #[cfg(feature = "rv32fd")]
    pub fn cimmd(instr: u16) -> i32 {
        (((instr & 0b0001_1100_0000_0000) >> 10) << 3 |
         ((instr & 0b0000_0000_0110_0000) >> 5) << 6) as i32
    }

    // Immediates (sign-extended).
    pub fn cimmi(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) as i32) << (31-12)) >> (31-5) |
        (((instr & 0b0000_0000_0111_1100) as i32) >> 2)
    }

    pub fn cimmui(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) as i32) << (31-12)) >> (31-17) |
        (((instr & 0b0000_0000_0111_1100) as i32) >> 2) << 12
    }

    pub fn cimm16sp(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) as i32) << (31-12)) >> (31-9) |
        (((instr & 0b0000_0000_0100_0000) as i32) >> 6) << 4 |
        (((instr & 0b0000_0000_0010_0000) as i32) >> 5) << 6 |
        (((instr & 0b0000_0000_0001_1000) as i32) >> 3) << 7 |
        (((instr & 0b0000_0000_0000_0100) as i32) >> 2) << 5
    }

    pub fn cimmj(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) as i32) << (31-12)) >> (31-11) |
        (((instr & 0b0000_1000_0000_0000) as i32) >> 11) << 4 |
        (((instr & 0b0000_0110_0000_0000) as i32) >> 9) << 8 |
        (((instr & 0b0000_0001_0000_0000) as i32) >> 8) << 10 |
        (((instr & 0b0000_0000_1000_0000) as i32) >> 7) << 6 |
        (((instr & 0b0000_0000_0100_0000) as i32) >> 6) << 7 |
        (((instr & 0b0000_0000_0011_1000) as i32) >> 3) << 1 |
        (((instr & 0b0000_0000_0000_0100) as i32) >> 2) << 5
    }

    pub fn cimmb(instr: u16) -> i32 {
        (((instr & 0b0001_0000_0000_0000) as i32) << (31-12)) >> (31-8) |
        (((instr & 0b0000_1100_0000_0000) as i32) >> 10) << 3 |
        (((instr & 0b0000_0000_0110_0000) as i32) >> 5) << 6 |
        (((instr & 0b0000_0000_0001_1000) as i32) >> 3) << 1 |
        (((instr & 0b0000_0000_0000_0100) as i32) >> 2) << 5
    }
}
#[cfg(feature = "rv32c")]
use self::rv32c::*;
