// The source files `interp.rs` and `op.rs` are generated from this file. They are based on the
// function declarations in this file that have a special `//%` comment. This comment contains the
// fields and values that should be matched on. In addition, the function argument names define
// fields that should be captured in the `Op` enum variant. Both of these are matched by name to
// functions defined in the `op` module.

use crate::cpu::op::Op;
use crate::cpu::types::{Clock, CpuError, CpuState, Memory, MemoryAccess};
#[cfg(feature = "rv32fd")]
use crate::softfloat::{self as sf, Sf32, Sf64};
#[cfg(feature = "rv32fd")]
use std::num::FpCategory;

type CpuExit = Result<(), CpuError>;

enum CsrAccess<'a> {
    Read(&'a mut u32),
    Write(u32),
}

/// The interpeter.
///
/// This struct simply combines a `CpuState`, `Memory` and `Clock`. An `Interp` instance can be
/// fleeting, and doesn't need to be kept around if the virtual CPU is paused, for example.
pub struct Interp<'s, 'm, 'c, M: 'm + Memory, C: 'c + Clock> {
    /// The CPU state.
    pub state: &'s mut CpuState,
    /// The memory implementation.
    pub mem: &'m mut M,
    /// The clock implementation.
    pub clock: &'c mut C,
    /// Size of the last instruction (2 or 4).
    instsz: u32,
}

impl<'s, 'm, 'c, M: 'm + Memory, C: 'c + Clock> Interp<'s, 'm, 'c, M, C> {
    /// Create a new interpreter.
    pub fn new(state: &'s mut CpuState, mem: &'m mut M, clock: &'c mut C) -> Self {
        Self { state, mem, clock, instsz: 4 }
    }

    /// Run continuously until execution stops, starting at the current PC address.
    ///
    /// Returns the stop reason and the instruction that caused the virtual CPU to stop. The
    /// instruction may be `None` if it failed to load or parse.
    pub fn run(&mut self) -> (CpuError, Option<Op>) {
        loop {
            if let Err(err) = self.step() {
                return err;
            }
        }
    }

    /// Step a single instruction, fetching it from the current PC address.
    ///
    /// Returns the parsed instruction that was executed. When the instruction stops the virtual
    /// CPU, additionally returns the stop reason. In the latter case, the instruction may be
    /// `None` if it failed to load or parse.
    pub fn step(&mut self) -> Result<Op, (CpuError, Option<Op>)> {
        // Increment counters.
        if !self.clock.check_quota() {
            return Err((CpuError::QuotaExceeded, None));
        }

        let op = match {
            #[cfg(feature = "rv32c")]
            {
                // Read the next instruction.
                let mut instr_lo: u16 = 0;
                if !self.mem.access(self.state.pc, MemoryAccess::Exec(&mut instr_lo)) {
                    return Err((CpuError::IllegalFetch, None));
                }

                // Parse into an `Op`.
                if (instr_lo & 3) == 3 {
                    let mut instr_hi: u16 = 0;
                    if !self.mem.access(self.state.pc + 2, MemoryAccess::Exec(&mut instr_hi)) {
                        return Err((CpuError::IllegalFetch, None));
                    }
                    self.instsz = 4;
                    Op::parse((instr_hi as u32) << 16 | (instr_lo as u32))
                } else {
                    self.instsz = 2;
                    Op::parse_c(instr_lo)
                }
            }
            #[cfg(not(feature = "rv32c"))]
            {
                // Read the next instruction.
                let mut instr: u32 = 0;
                if !self.mem.access(self.state.pc, MemoryAccess::Exec(&mut instr)) {
                    return Err((CpuError::IllegalFetch, None));
                }

                // Parse into an `Op`.
                Op::parse(instr)
            }
        } {
            Some(op) => op,
            None => return Err((CpuError::IllegalInstruction, None)),
        };

        // Dispatch the instruction.
        let res = match op {
            //% dispatch
        };

        // Increment counters.
        self.clock.progress(&op);

        // Attach the `Op` to the result.
        match res {
            Ok(_) => Ok(op),
            Err(err) => Err((err, Some(op))),
        }
    }

    /// Read a value from or write a value to a CSR.
    fn access_csr(&mut self, id: u32, access: CsrAccess) -> bool {
        match id {
            0x001 => { // fflags
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = self.state.fcsr & 0x1f;
                        true
                    },
                    CsrAccess::Write(value) => {
                        self.state.fcsr = (self.state.fcsr & 0xffff_ffe0) + (value & 0x1f);
                        true
                    },
                }
            },
            0x002 => { // frm
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = (self.state.fcsr & 0xe0) >> 5;
                        true
                    },
                    CsrAccess::Write(value) => {
                        self.state.fcsr = (self.state.fcsr & 0xffff_ff1f) + ((value & 0x7) << 5);
                        true
                    },
                }
            },
            0x003 => { // fcsr
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = self.state.fcsr & 0xff;
                        true
                    },
                    CsrAccess::Write(value) => {
                        self.state.fcsr = (self.state.fcsr & 0xffff_ff00) + (value & 0xff);
                        true
                    },
                }
            },
            0xC00 => { // cycle
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = self.clock.read_cycle() as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            0xC80 => { // cycleh
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = (self.clock.read_cycle() >> 32) as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            0xC01 => { // time
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = self.clock.read_time() as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            0xC81 => { // timeh
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = (self.clock.read_time() >> 32) as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            0xC02 => { // instret
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = self.clock.read_instret() as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            0xC82 => { // instreth
                match access {
                    CsrAccess::Read(dest) => {
                        *dest = (self.clock.read_instret() >> 32) as u32;
                        true
                    },
                    CsrAccess::Write(_) => {
                        true
                    },
                }
            },
            _ => false,
        }
    }

    // 
    // RV32I Base Integer Instruction Set
    //

    //% opcode=011_0111
    fn lui(&mut self, rd: usize, u_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            u_imm as u32
        });
        end_op!(self)
    }

    //% opcode=001_0111
    fn auipc(&mut self, rd: usize, u_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            self.state.pc.wrapping_add(u_imm as u32)
        });
        end_op!(self)
    }

    //% opcode=110_1111
    fn jal(&mut self, rd: usize, j_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            self.state.pc.wrapping_add(self.instsz)
        });
        end_jump_op!(self, {
            self.state.pc.wrapping_add(j_imm as u32)
        })
    }

    //% opcode=110_0111 funct3=000
    fn jalr(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let dst_base = self.state.x[rs1];
        write_rd!(self, rd, {
            self.state.pc.wrapping_add(self.instsz)
        });
        end_jump_op!(self, {
            dst_base.wrapping_add(i_imm as u32)
        })
    }

    //% opcode=110_0011 funct3=000
    fn beq(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if self.state.x[rs1] == self.state.x[rs2] {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=110_0011 funct3=001
    fn bne(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if self.state.x[rs1] != self.state.x[rs2] {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=110_0011 funct3=100
    fn blt(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if (self.state.x[rs1] as i32) < (self.state.x[rs2] as i32) {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=110_0011 funct3=101
    fn bge(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if (self.state.x[rs1] as i32) >= (self.state.x[rs2] as i32) {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=110_0011 funct3=110
    fn bltu(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if self.state.x[rs1] < self.state.x[rs2] {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=110_0011 funct3=111
    fn bgeu(&mut self, rs1: usize, rs2: usize, b_imm: i32) -> CpuExit {
        if self.state.x[rs1] >= self.state.x[rs2] {
            end_branch_op!(self, b_imm)
        } else {
            end_op!(self)
        }
    }

    //% opcode=000_0011 funct3=000
    fn lb(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: i8 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            write_rd!(self, rd, { value as u32 });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=000_0011 funct3=001
    fn lh(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: i16 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            write_rd!(self, rd, { value as u32 });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=000_0011 funct3=010
    fn lw(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: u32 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            write_rd!(self, rd, { value as u32 });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=000_0011 funct3=100
    fn lbu(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: u8 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            write_rd!(self, rd, { value as u32 });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=000_0011 funct3=101
    fn lhu(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: u16 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            write_rd!(self, rd, { value as u32 });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_0011 funct3=000
    fn sb(&mut self, rs1: usize, rs2: usize, s_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(s_imm as u32);
        let value = self.state.x[rs2] as u8;
        if self.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_0011 funct3=001
    fn sh(&mut self, rs1: usize, rs2: usize, s_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(s_imm as u32);
        let value = self.state.x[rs2] as u16;
        if self.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_0011 funct3=010
    fn sw(&mut self, rs1: usize, rs2: usize, s_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(s_imm as u32);
        let value = self.state.x[rs2];
        if self.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=001_0011 funct3=000
    fn addi(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_add(i_imm as u32)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=010
    fn slti(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            if (self.state.x[rs1] as i32) < i_imm { 1 } else { 0 }
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=011
    fn sltiu(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            if self.state.x[rs1] < i_imm as u32 { 1 } else { 0 }
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=100
    fn xori(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            (self.state.x[rs1] ^ i_imm as u32)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=110
    fn ori(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            (self.state.x[rs1] | i_imm as u32)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=111
    fn andi(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        write_rd!(self, rd, {
            (self.state.x[rs1] & i_imm as u32)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=001 shtype=000_0000
    fn slli(&mut self, rd: usize, rs1: usize, shamt: u32) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_shl(shamt)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=101 shtype=000_0000
    fn srli(&mut self, rd: usize, rs1: usize, shamt: u32) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_shr(shamt)
        });
        end_op!(self)
    }

    //% opcode=001_0011 funct3=101 shtype=010_0000
    fn srai(&mut self, rd: usize, rs1: usize, shamt: u32) -> CpuExit {
        write_rd!(self, rd, {
            ((self.state.x[rs1] as i32).wrapping_shr(shamt) as u32)
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=000
    fn add(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_add(self.state.x[rs2])
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=001
    fn sll(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            (self.state.x[rs1]).wrapping_shl(self.state.x[rs2])
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=010
    fn slt(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            if (self.state.x[rs1] as i32) < (self.state.x[rs2] as i32) { 1 } else { 0 }
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=011
    fn sltu(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            if self.state.x[rs1] < self.state.x[rs2] { 1 } else { 0 }
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=100
    fn xor(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1] ^ self.state.x[rs2]
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=101
    fn srl(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_shr(self.state.x[rs2])
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=110
    fn or(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1] | self.state.x[rs2]
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0000 funct3=111
    fn and(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1] & self.state.x[rs2]
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=010_0000 funct3=000
    fn sub(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_sub(self.state.x[rs2])
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=010_0000 funct3=101
    fn sra(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            ((self.state.x[rs1] as i32).wrapping_shr(self.state.x[rs2])) as u32
        });
        end_op!(self)
    }

    //% opcode=000_1111 funct3=000 rd=0_0000 rs1=0_0000 unused1=0000
    fn fence(&mut self, _pred: u32, _succ: u32) -> CpuExit {
        end_op!(self)
    }

    //% opcode=000_1111 funct3=001 rd=0_0000 rs1=0_0000 unused1=0000
    fn fence_i(&mut self) -> CpuExit {
        end_op!(self)
    }

    //% opcode=111_0011 funct3=000 funct12=0000_0000_0000 rd=0_0000 rs1=0_0000
    fn ecall(&mut self) -> CpuExit {
        end_op!(self, Ecall)
    }

    //% opcode=111_0011 funct3=000 funct12=0000_0000_0001 rd=0_0000 rs1=0_0000
    fn ebreak(&mut self) -> CpuExit {
        end_op!(self, Ebreak)
    }

    //% opcode=111_0011 funct3=001
    fn csrrw(&mut self, rd: usize, rs1: usize, csr: u32) -> CpuExit {
        let new = self.state.x[rs1];

        write_rd!(self, rd, {
            let mut old: u32 = 0;
            if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
                end_op!(self, IllegalInstruction);
            }
            old
        });

        if !self.access_csr(csr, CsrAccess::Write(new)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //% opcode=111_0011 funct3=010
    fn csrrs(&mut self, rd: usize, rs1: usize, csr: u32) -> CpuExit {
        let mask = self.state.x[rs1];

        let mut old: u32 = 0;
        if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
            end_op!(self, IllegalInstruction);
        }
        write_rd!(self, rd, { old });

        if rs1 != 0 && !self.access_csr(csr, CsrAccess::Write(old | mask)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //% opcode=111_0011 funct3=011
    fn csrrc(&mut self, rd: usize, rs1: usize, csr: u32) -> CpuExit {
        let mask = self.state.x[rs1];

        let mut old: u32 = 0;
        if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
            end_op!(self, IllegalInstruction);
        }
        write_rd!(self, rd, { old });

        if rs1 != 0 && !self.access_csr(csr, CsrAccess::Write(old & !mask)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //% opcode=111_0011 funct3=101
    fn csrrwi(&mut self, rd: usize, zimm: u32, csr: u32) -> CpuExit {
        write_rd!(self, rd, {
            let mut old: u32 = 0;
            if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
                end_op!(self, IllegalInstruction);
            }
            old
        });

        if !self.access_csr(csr, CsrAccess::Write(zimm)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //% opcode=111_0011 funct3=110
    fn csrrsi(&mut self, rd: usize, zimm: u32, csr: u32) -> CpuExit {
        let mut old: u32 = 0;
        if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
            end_op!(self, IllegalInstruction);
        }
        write_rd!(self, rd, { old });

        if !self.access_csr(csr, CsrAccess::Write(old | zimm)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //% opcode=111_0011 funct3=111
    fn csrrci(&mut self, rd: usize, zimm: u32, csr: u32) -> CpuExit {
        let mut old: u32 = 0;
        if !self.access_csr(csr, CsrAccess::Read(&mut old)) {
            end_op!(self, IllegalInstruction);
        }
        write_rd!(self, rd, { old });

        if !self.access_csr(csr, CsrAccess::Write(old & !zimm)) {
            end_op!(self, IllegalInstruction);
        }

        end_op!(self)
    }

    //
    // "M" Standard Extension for Integer Multiplication and Division
    //

    //% opcode=011_0011 funct7=000_0001 funct3=000
    fn mul(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            self.state.x[rs1].wrapping_mul(self.state.x[rs2])
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=001
    fn mulh(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let x = (self.state.x[rs1] as i32) as i64;
            let y = (self.state.x[rs2] as i32) as i64;
            (x.wrapping_mul(y) >> 32) as u32
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=010
    fn mulhsu(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let x = (self.state.x[rs1] as i32) as i64;
            let y = self.state.x[rs2] as i64;
            (x.wrapping_mul(y) >> 32) as u32
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=011
    fn mulhu(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let x = self.state.x[rs1] as u64;
            let y = self.state.x[rs2] as u64;
            (x.wrapping_mul(y) >> 32) as u32
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=100
    fn div(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let y = self.state.x[rs2] as i32;
            if y == 0 {
                0xffff_ffff
            } else {
                let x = self.state.x[rs1] as i32;
                x.wrapping_div(y) as u32
            }
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=101
    fn divu(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let y = self.state.x[rs2];
            if y == 0 {
                0xffff_ffff
            } else {
                self.state.x[rs1].wrapping_div(y) as u32
            }
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=110
    fn rem(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let y = self.state.x[rs2] as i32;
            if y == 0 {
                self.state.x[rs1]
            } else {
                let x = self.state.x[rs1] as i32;
                x.wrapping_rem(y) as u32
            }
        });
        end_op!(self)
    }

    //% opcode=011_0011 funct7=000_0001 funct3=111
    fn remu(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        write_rd!(self, rd, {
            let y = self.state.x[rs2];
            if y == 0 {
                self.state.x[rs1]
            } else {
                self.state.x[rs1].wrapping_rem(y) as u32
            }
        });
        end_op!(self)
    }

    //
    // "A" Standard Extension for Atomic Instructions
    //

    //% opcode=010_1111 funct3=010 funct5=0_0010 rs2=0_0000
    fn lr_w(&mut self, rd: usize, rs1: usize, _aq: bool, _rl: bool) -> CpuExit {
        let addr = self.state.x[rs1];
        let mut value: u32 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            self.state.reservation = Some(addr);
            write_rd!(self, rd, { value });
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_1111 funct3=010 funct5=0_0011
    fn sc_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        let addr = self.state.x[rs1];
        if self.state.reservation == Some(addr) {
            let value = self.state.x[rs2];
            if self.mem.access(addr, MemoryAccess::Store(value)) {
                write_rd!(self, rd, { 0 });
                self.state.reservation = None;
                end_op!(self)
            } else {
                end_op!(self, IllegalAccess)
            }
        } else {
            write_rd!(self, rd, { 1 });
            end_op!(self)
        }
    }

    //% opcode=010_1111 funct3=010 funct5=0_0001
    fn amoswap_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rs2]
        })
    }

    //% opcode=010_1111 funct3=010 funct5=0_0000
    fn amoadd_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd].wrapping_add(self.state.x[rs2])
        })
    }

    //% opcode=010_1111 funct3=010 funct5=0_0100
    fn amoxor_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd] ^ self.state.x[rs2]
        })
    }

    //% opcode=010_1111 funct3=010 funct5=0_1100
    fn amoand_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd] & self.state.x[rs2]
        })
    }

    //% opcode=010_1111 funct3=010 funct5=0_1000
    fn amoor_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd] | self.state.x[rs2]
        })
    }

    //% opcode=010_1111 funct3=010 funct5=1_0000
    fn amomin_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            (self.state.x[rd] as i32).min(self.state.x[rs2] as i32) as u32
        })
    }

    //% opcode=010_1111 funct3=010 funct5=1_0100
    fn amomax_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            (self.state.x[rd] as i32).max(self.state.x[rs2] as i32) as u32
        })
    }

    //% opcode=010_1111 funct3=010 funct5=1_1000
    fn amominu_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd].min(self.state.x[rs2])
        })
    }

    //% opcode=010_1111 funct3=010 funct5=1_1100
    fn amomaxu_w(&mut self, rd: usize, rs1: usize, rs2: usize, _aq: bool, _rl: bool) -> CpuExit {
        amo!(self, rd, rs1, {
            self.state.x[rd].max(self.state.x[rs2])
        })
    }

    //
    // "F" Standard Extension for Single-Precision Floating-Point
    //
    //f{

    //% opcode=000_0111 funct3=010
    fn flw(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: u32 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            self.state.f[rd] = Sf64::from(Sf32(value));
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_0111 funct3=010
    fn fsw(&mut self, rs1: usize, rs2: usize, s_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(s_imm as u32);
        let value = Sf32::from(self.state.f[rs2]).0;
        if self.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=100_0011 funct2=00
    fn fmadd_s(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_mulAdd(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2]),
                Sf32::from(self.state.f[rs3])
            ))
        } })
    }

    //% opcode=100_0111 funct2=00
    fn fmsub_s(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_mulAdd(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2]),
                Sf32::from(self.state.f[rs3]).negate()
            ))
        } })
    }

    //% opcode=100_1011 funct2=00
    fn fnmsub_s(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_mulAdd(
                Sf32::from(self.state.f[rs1]).negate(),
                Sf32::from(self.state.f[rs2]),
                Sf32::from(self.state.f[rs3])
            ))
        } })
    }

    //% opcode=100_1111 funct2=00
    fn fnmadd_s(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_mulAdd(
                Sf32::from(self.state.f[rs1]).negate(),
                Sf32::from(self.state.f[rs2]),
                Sf32::from(self.state.f[rs3]).negate()
            ))
        } })
    }

    //% opcode=101_0011 funct7=000_0000
    fn fadd_s(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_add(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2])
            ))
        } })
    }

    //% opcode=101_0011 funct7=000_0100
    fn fsub_s(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_sub(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2])
            ))
        } })
    }

    //% opcode=101_0011 funct7=000_1000
    fn fmul_s(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_mul(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2])
            ))
        } })
    }

    //% opcode=101_0011 funct7=000_1100
    fn fdiv_s(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_div(
                Sf32::from(self.state.f[rs1]),
                Sf32::from(self.state.f[rs2])
            ))
        } })
    }

    //% opcode=101_0011 funct7=010_1100 rs2=0_0000
    fn fsqrt_s(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::f32_sqrt(
                Sf32::from(self.state.f[rs1])
            ))
        } })
    }

    //% opcode=101_0011 funct7=001_0000 funct3=000
    fn fsgnj_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let a = Sf32::from(self.state.f[rs1]).0;
        let b = Sf32::from(self.state.f[rs2]).0;
        self.state.f[rd] = Sf64::from(Sf32((a & 0x7fff_ffff) | (b & 0x8000_0000)));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0000 funct3=001
    fn fsgnjn_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let a = Sf32::from(self.state.f[rs1]).0;
        let b = Sf32::from(self.state.f[rs2]).0;
        self.state.f[rd] = Sf64::from(Sf32((a & 0x7fff_ffff) | (!b & 0x8000_0000)));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0000 funct3=010
    fn fsgnjx_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let a = Sf32::from(self.state.f[rs1]).0;
        let b = Sf32::from(self.state.f[rs2]).0;
        self.state.f[rd] = Sf64::from(Sf32(a ^ (b & 0x8000_0000)));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0100 funct3=000
    fn fmin_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_calc!(self, rd, { unsafe {
            let a = f32::from(self.state.f[rs1]);
            let b = f32::from(self.state.f[rs2]);

            if sf::f32_is_signaling_nan(Sf32::from(a)) || sf::f32_is_signaling_nan(Sf32::from(b)) {
                sf::raise_flags(sf::FLAG_INVALID);
            }

            Sf64::from(match (a.classify(), b.classify()) {
                (FpCategory::Nan, FpCategory::Nan) => f32::from(Sf32::NAN),
                (FpCategory::Nan, _) => b,
                (_, FpCategory::Nan) => a,
                (FpCategory::Zero, FpCategory::Zero) => {
                    if a.is_sign_negative() { a } else { b }
                },
                _ => f32::min(a, b),
            })
        } })
    }

    //% opcode=101_0011 funct7=001_0100 funct3=001
    fn fmax_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_calc!(self, rd, { unsafe {
            let a = f32::from(self.state.f[rs1]);
            let b = f32::from(self.state.f[rs2]);

            if sf::f32_is_signaling_nan(Sf32::from(a)) || sf::f32_is_signaling_nan(Sf32::from(b)) {
                sf::raise_flags(sf::FLAG_INVALID);
            }

            Sf64::from(match (a.classify(), b.classify()) {
                (FpCategory::Nan, FpCategory::Nan) => f32::from(Sf32::NAN),
                (FpCategory::Nan, _) => b,
                (_, FpCategory::Nan) => a,
                (FpCategory::Zero, FpCategory::Zero) => {
                    if a.is_sign_positive() { a } else { b }
                },
                _ => f32::max(a, b),
            })
        } })
    }

    //% opcode=101_0011 funct7=110_0000 rs2=0_0000
    fn fcvt_w_s(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_wrap!(self, rm, {
            write_rd!(self, rd, { unsafe {
                sf::f32_to_i32(
                    Sf32::from(self.state.f[rs1]),
                    sf::get_rounding_mode(),
                    true
                ) as u32
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=110_0000 rs2=0_0001
    fn fcvt_wu_s(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_wrap!(self, rm, {
            write_rd!(self, rd, { unsafe {
                sf::f32_to_u32(
                    Sf32::from(self.state.f[rs1]),
                    sf::get_rounding_mode(),
                    true
                )
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=111_0000 funct3=000 rs2=0_0000
    fn fmv_x_w(&mut self, rd: usize, rs1: usize) -> CpuExit {
        self.state.x[rd] = Sf32::from(self.state.f[rs1]).0;
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0000 funct3=010
    fn feq_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f32_eq(
                    Sf32::from(self.state.f[rs1]),
                    Sf32::from(self.state.f[rs2])
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0000 funct3=001
    fn flt_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f32_lt(
                    Sf32::from(self.state.f[rs1]),
                    Sf32::from(self.state.f[rs2])
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0000 funct3=000
    fn fle_s(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f32_le(
                    Sf32::from(self.state.f[rs1]),
                    Sf32::from(self.state.f[rs2])
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=111_0000 funct3=001 rs2=0_0000
    fn fclass_s(&mut self, rd: usize, rs1: usize) -> CpuExit {
        let v = f32::from(self.state.f[rs1]);
        write_rd!(self, rd, { match v.classify() {
            FpCategory::Nan => {
                if unsafe { sf::f32_is_signaling_nan(Sf32::from(v)) } { 0b01_0000_0000 } else { 0b10_0000_0000 }
            },
            FpCategory::Infinite => {
                if v.is_sign_positive() { 0b00_1000_0000 } else { 0b00_0000_0001 }
            },
            FpCategory::Zero => {
                if v.is_sign_positive() { 0b00_0001_0000 } else { 0b00_0000_1000 }
            },
            FpCategory::Subnormal => {
                if v.is_sign_positive() { 0b00_0010_0000 } else { 0b00_0000_0100 }
            },
            FpCategory::Normal => {
                if v.is_sign_positive() { 0b00_0100_0000 } else { 0b00_0000_0010 }
            },
        } });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=110_1000 rs2=0_0000
    fn fcvt_s_w(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::i32_to_f32(self.state.x[rs1] as i32))
        } });
    }

    //% opcode=101_0011 funct7=110_1000 rs2=0_0001
    fn fcvt_s_wu(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            Sf64::from(sf::u32_to_f32(self.state.x[rs1]))
        } });
    }

    //% opcode=101_0011 funct7=111_1000 funct3=000 rs2=0_0000
    fn fmv_w_x(&mut self, rd: usize, rs1: usize) -> CpuExit {
        self.state.f[rd] = Sf64::from(Sf32(self.state.x[rs1]));
        end_op!(self)
    }

    //
    // "D" Standard Extension for Double-Precision Floating-Point
    //

    //% opcode=000_0111 funct3=011
    fn fld(&mut self, rd: usize, rs1: usize, i_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(i_imm as u32);
        let mut value: u64 = 0;
        if self.mem.access(addr, MemoryAccess::Load(&mut value)) {
            self.state.f[rd] = Sf64(value);
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=010_0111 funct3=011
    fn fsd(&mut self, rs1: usize, rs2: usize, s_imm: i32) -> CpuExit {
        let addr = self.state.x[rs1].wrapping_add(s_imm as u32);
        let value = self.state.f[rs2].0;
        if self.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!(self)
        } else {
            end_op!(self, IllegalAccess)
        }
    }

    //% opcode=100_0011 funct2=01
    fn fmadd_d(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_mulAdd(
                self.state.f[rs1],
                self.state.f[rs2],
                self.state.f[rs3]
            )
        } })
    }

    //% opcode=100_0111 funct2=01
    fn fmsub_d(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_mulAdd(
                self.state.f[rs1],
                self.state.f[rs2],
                self.state.f[rs3].negate()
            )
        } })
    }

    //% opcode=100_1011 funct2=01
    fn fnmsub_d(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_mulAdd(
                self.state.f[rs1].negate(),
                self.state.f[rs2],
                self.state.f[rs3]
            )
        } })
    }

    //% opcode=100_1111 funct2=01
    fn fnmadd_d(&mut self, rd: usize, rs1: usize, rs2: usize, rs3: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_mulAdd(
                self.state.f[rs1].negate(),
                self.state.f[rs2],
                self.state.f[rs3].negate()
            )
        } })
    }

    //% opcode=101_0011 funct7=000_0001
    fn fadd_d(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_add(
                self.state.f[rs1],
                self.state.f[rs2]
            )
        } })
    }

    //% opcode=101_0011 funct7=000_0101
    fn fsub_d(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_sub(
                self.state.f[rs1],
                self.state.f[rs2]
            )
        } })
    }

    //% opcode=101_0011 funct7=000_1001
    fn fmul_d(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_mul(
                self.state.f[rs1],
                self.state.f[rs2]
            )
        } })
    }

    //% opcode=101_0011 funct7=000_1101
    fn fdiv_d(&mut self, rd: usize, rs1: usize, rs2: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_div(
                self.state.f[rs1],
                self.state.f[rs2]
            )
        } })
    }

    //% opcode=101_0011 funct7=010_1101 rs2=0_0000
    fn fsqrt_d(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::f64_sqrt(
                self.state.f[rs1]
            )
        } })
    }

    //% opcode=101_0011 funct7=001_0001 funct3=000
    fn fsgnj_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let Sf64(a) = self.state.f[rs1];
        let Sf64(b) = self.state.f[rs2];
        self.state.f[rd] = Sf64((a & 0x7fff_ffff_ffff_ffff) | (b & 0x8000_0000_0000_0000));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0001 funct3=001
    fn fsgnjn_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let Sf64(a) = self.state.f[rs1];
        let Sf64(b) = self.state.f[rs2];
        self.state.f[rd] = Sf64((a & 0x7fff_ffff_ffff_ffff) | (!b & 0x8000_0000_0000_0000));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0001 funct3=010
    fn fsgnjx_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        let Sf64(a) = self.state.f[rs1];
        let Sf64(b) = self.state.f[rs2];
        self.state.f[rd] = Sf64(a ^ (b & 0x8000_0000_0000_0000));
        end_op!(self)
    }

    //% opcode=101_0011 funct7=001_0101 funct3=000
    fn fmin_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_calc!(self, rd, { unsafe {
            let a = f64::from(self.state.f[rs1]);
            let b = f64::from(self.state.f[rs2]);

            if sf::f64_is_signaling_nan(Sf64::from(a)) || sf::f64_is_signaling_nan(Sf64::from(b)) {
                sf::raise_flags(sf::FLAG_INVALID);
            }

            Sf64::from(match (a.classify(), b.classify()) {
                (FpCategory::Nan, FpCategory::Nan) => f64::from(Sf64::NAN),
                (FpCategory::Nan, _) => b,
                (_, FpCategory::Nan) => a,
                (FpCategory::Zero, FpCategory::Zero) => {
                    if a.is_sign_negative() { a } else { b }
                },
                _ => f64::min(a, b),
            })
        } })
    }

    //% opcode=101_0011 funct7=001_0101 funct3=001
    fn fmax_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_calc!(self, rd, { unsafe {
            let a = f64::from(self.state.f[rs1]);
            let b = f64::from(self.state.f[rs2]);

            if sf::f64_is_signaling_nan(Sf64::from(a)) || sf::f64_is_signaling_nan(Sf64::from(b)) {
                sf::raise_flags(sf::FLAG_INVALID);
            }

            Sf64::from(match (a.classify(), b.classify()) {
                (FpCategory::Nan, FpCategory::Nan) => f64::from(Sf64::NAN),
                (FpCategory::Nan, _) => b,
                (_, FpCategory::Nan) => a,
                (FpCategory::Zero, FpCategory::Zero) => {
                    if a.is_sign_positive() { a } else { b }
                },
                _ => f64::max(a, b),
            })
        } })
    }

    //% opcode=101_0011 funct7=110_0001 rs2=0_0000
    fn fcvt_w_d(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_wrap!(self, rm, {
            write_rd!(self, rd, { unsafe {
                sf::f64_to_i32(
                    self.state.f[rs1],
                    sf::get_rounding_mode(),
                    true
                ) as u32
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=110_0001 rs2=0_0001
    fn fcvt_wu_d(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_wrap!(self, rm, {
            write_rd!(self, rd, { unsafe {
                sf::f64_to_u32(
                    self.state.f[rs1],
                    sf::get_rounding_mode(),
                    true
                )
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0001 funct3=010
    fn feq_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f64_eq(
                    self.state.f[rs1],
                    self.state.f[rs2]
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0001 funct3=001
    fn flt_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f64_lt(
                    self.state.f[rs1],
                    self.state.f[rs2]
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=101_0001 funct3=000
    fn fle_d(&mut self, rd: usize, rs1: usize, rs2: usize) -> CpuExit {
        sf_wrap!(self, {
            write_rd!(self, rd, { unsafe {
                let res = sf::f64_le(
                    self.state.f[rs1],
                    self.state.f[rs2]
                );
                if res { 1 } else { 0 }
            } });
        });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=111_0001 funct3=001 rs2=0_0000
    fn fclass_d(&mut self, rd: usize, rs1: usize) -> CpuExit {
        let v = f64::from(self.state.f[rs1]);
        write_rd!(self, rd, { match v.classify() {
            FpCategory::Nan => {
                if unsafe { sf::f64_is_signaling_nan(Sf64::from(v)) } { 0b01_0000_0000 } else { 0b10_0000_0000 }
            },
            FpCategory::Infinite => {
                if v.is_sign_positive() { 0b00_1000_0000 } else { 0b00_0000_0001 }
            },
            FpCategory::Zero => {
                if v.is_sign_positive() { 0b00_0001_0000 } else { 0b00_0000_1000 }
            },
            FpCategory::Subnormal => {
                if v.is_sign_positive() { 0b00_0010_0000 } else { 0b00_0000_0100 }
            },
            FpCategory::Normal => {
                if v.is_sign_positive() { 0b00_0100_0000 } else { 0b00_0000_0010 }
            },
        } });
        end_op!(self)
    }

    //% opcode=101_0011 funct7=110_1001 rs2=0_0000
    fn fcvt_d_w(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::i32_to_f64(self.state.x[rs1] as i32)
        } });
    }

    //% opcode=101_0011 funct7=110_1001 rs2=0_0001
    fn fcvt_d_wu(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            sf::u32_to_f64(self.state.x[rs1])
        } });
    }

    //% opcode=101_0011 funct7=010_0000 rs2=0_0001
    fn fcvt_s_d(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            let v = self.state.f[rs1];
            if f64::from(v).is_nan() {
                Sf64::from(Sf32::NAN)
            } else {
                Sf64::from(sf::f64_to_f32(v))
            }
        } });
    }

    //% opcode=101_0011 funct7=010_0001 rs2=0_0000
    fn fcvt_d_s(&mut self, rd: usize, rs1: usize, rm: u32) -> CpuExit {
        sf_calc!(self, rm, rd, { unsafe {
            let v = Sf32::from(self.state.f[rs1]);
            if f32::from(v).is_nan() {
                Sf64::NAN
            } else {
                sf::f32_to_f64(v)
            }
        } });
    }
    //f}

    //
    // "C" Standard Extension for Compressed Instructions, Version 2.0
    //

    //% cquad=00 cfunct3=000 cimm4spn=0000_0000
    //    name=c_illegal decomp=illegal
    //
    //% cquad=00 cfunct3=000 cimm4spn=_
    //    name=c_addi4spn decomp=addi rd=crs2q rs1=crsp i_imm=cimm4spn
    //
    //% cquad=00 cfunct3=010
    //    name=c_lw decomp=lw rd=crs2q rs1=crs1rdq i_imm=cimmw
    //
    //% cquad=00 cfunct3=110
    //    name=c_sw decomp=sw rs1=crs1rdq rs2=crs2q s_imm=cimmw
    //
    //% cquad=01 cfunct3=000
    //    name=c_addi decomp=addi rd=crs1rd rs1=crs1rd i_imm=cimmi
    //
    //% cquad=01 cfunct3=001
    //    name=c_jal decomp=jal rd=crra j_imm=cimmj
    //
    //% cquad=01 cfunct3=010
    //    name=c_li decomp=addi rd=crs1rd rs1=crx0 i_imm=cimmi
    //
    //% cquad=01 cfunct3=011 crs1rd=0_0010
    //    name=c_addi16sp decomp=addi rd=crsp rs1=crsp i_imm=cimm16sp
    //
    //% cquad=01 cfunct3=011 crs1rd=_
    //    name=c_lui decomp=lui rd=crs1rd u_imm=cimmui
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=00
    //    name=c_srli decomp=srli rd=crs1rdq rs1=crs1rdq shamt=cimmsh6
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=01
    //    name=c_srai decomp=srai rd=crs1rdq rs1=crs1rdq shamt=cimmsh6
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=10
    //    name=c_andi decomp=andi rd=crs1rdq rs1=crs1rdq i_imm=cimmi
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=11 cfunct4_l0=0 crs2_h2=00
    //    name=c_sub decomp=sub rd=crs1rdq rs1=crs1rdq rs2=crs2q
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=11 cfunct4_l0=0 crs2_h2=01
    //    name=c_xor decomp=xor rd=crs1rdq rs1=crs1rdq rs2=crs2q
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=11 cfunct4_l0=0 crs2_h2=10
    //    name=c_or decomp=or rd=crs1rdq rs1=crs1rdq rs2=crs2q
    //
    //% cquad=01 cfunct3=100 crs1rd_h2=11 cfunct4_l0=0 crs2_h2=11
    //    name=c_and decomp=and rd=crs1rdq rs1=crs1rdq rs2=crs2q
    //
    //% cquad=01 cfunct3=101
    //    name=c_j decomp=jal rd=crx0 j_imm=cimmj
    //
    //% cquad=01 cfunct3=110
    //    name=c_beqz decomp=beq rs1=crs1rdq rs2=crx0 b_imm=cimmb
    //
    //% cquad=01 cfunct3=111
    //    name=c_bnez decomp=bne rs1=crs1rdq rs2=crx0 b_imm=cimmb
    //
    //% cquad=10 cfunct3=000
    //    name=c_slli decomp=slli rd=crs1rd rs1=crs1rd shamt=cimmsh6
    //
    //% cquad=10 cfunct3=010
    //    name=c_lwsp decomp=lw rd=crs1rd rs1=crsp i_imm=cimmlwsp
    //
    //% cquad=10 cfunct3=100 cfunct4_l0=0 crs2=0_0000
    //    name=c_jr decomp=jalr rd=crx0 rs1=crs1rd i_imm=czero
    //
    //% cquad=10 cfunct3=100 cfunct4_l0=0 crs2=_
    //    name=c_mv decomp=addi rd=crs1rd rs1=crs2 i_imm=czero
    //
    //% cquad=10 cfunct3=100 cfunct4_l0=1 crs2=0_0000 crs1rd=0_0000
    //    name=c_ebreak decomp=ebreak
    //
    //% cquad=10 cfunct3=100 cfunct4_l0=1 crs2=0_0000 crs1rd=_
    //    name=c_jalr decomp=jalr rd=crra rs1=crs1rd i_imm=czero
    //
    //% cquad=10 cfunct3=100 cfunct4_l0=1 crs2=_
    //    name=c_add decomp=add rd=crs1rd rs1=crs1rd rs2=crs2
    //
    //% cquad=10 cfunct3=110
    //    name=c_swsp decomp=sw rs1=crsp rs2=crs2 s_imm=cimmswsp
    //
    //f{
    //% cquad=00 cfunct3=001
    //    name=c_fld decomp=fld rd=crs2q rs1=crs1rdq i_imm=cimmd
    //
    //% cquad=00 cfunct3=011
    //    name=c_flw decomp=flw rd=crs2q rs1=crs1rdq i_imm=cimmw
    //
    //% cquad=00 cfunct3=101
    //    name=c_fsd decomp=fsd rs1=crs1rdq rs2=crs2q s_imm=cimmd
    //
    //% cquad=00 cfunct3=111
    //    name=c_fsw decomp=fsw rs1=crs1rdq rs2=crs2q s_imm=cimmw
    //
    //% cquad=10 cfunct3=001
    //    name=c_fldsp decomp=fld rd=crs1rd rs1=crsp i_imm=cimmldsp
    //
    //% cquad=10 cfunct3=011
    //    name=c_flwsp decomp=flw rd=crs1rd rs1=crsp i_imm=cimmlwsp
    //
    //% cquad=10 cfunct3=101
    //    name=c_fsdsp decomp=fsd rs1=crsp rs2=crs2 s_imm=cimmsdsp
    //
    //% cquad=10 cfunct3=111
    //    name=c_fswsp decomp=fsw rs1=crsp rs2=crs2 s_imm=cimmswsp
    //f}
}
