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
}

//
// Matching fields.
//

fn opcode(instr: u32) -> u32 {
    instr & 0b0000_0000_0000_0000_0000_0000_0111_1111
}

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
