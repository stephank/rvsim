use ::cpu::op::Op;
#[cfg(feature = "rv32fd")]
use ::softfloat::Sf64;
use std::mem::size_of;

/// Statuses with which virtual CPU execution may stop.
///
/// Some of these may be recovered from. For all errors, the effects are documented.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum CpuError {
    /// Tried to branch or jump to an unaligned address.
    ///
    /// This error is typically fatal. `pc` is unaltered, but the jump or branch may have partially
    /// altered state.
    MisalignedFetch,

    /// Tried to fetch the next instruction from a bad address.
    ///
    /// This error is typically fatal. State is unaltered.
    IllegalFetch,

    /// Tried to execute an invalid instruction.
    ///
    /// This error is typically fatal. State is unaltered.
    IllegalInstruction,

    /// Tried to access an invalid address.
    ///
    /// This error is typically fatal. `pc` is advanced to the next instruction, but the
    /// instruction may have also partially altered state. This is especially true for atomic
    /// instructions or loads/stores that have side-effects.
    IllegalAccess,

    /// Tried to access a misaligned address.
    ///
    /// This error is typically fatal. `pc` is advanced to the next instruction, no other state is
    /// altered.
    MisalignedAccess,

    /// Encountered an ECALL instruction.
    ///
    /// This is typically handled by the caller and resumed from. `pc` is advanced to the next
    /// instruction, no other state is altered.
    Ecall,

    /// Encountered an EBREAK instruction.
    ///
    /// This is typically handled by the caller and resumed from. `pc` is advanced to the next
    /// instruction, no other state is altered.
    Ebreak,

    /// The `Clock` indicated the execution quota was exceeded.
    ///
    /// This is typically handled by the caller and resumed from. State is unaltered.
    QuotaExceeded,
}

/// Struct containing all virtual CPU state.
///
/// With the `serialize` crate feature, this structure is serializable using Serde.
#[derive(Clone,Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize,Deserialize))]
pub struct CpuState {
    /// Integer registers.
    pub x: [u32; 32],

    /// Floating-point registers.
    #[cfg(feature = "rv32fd")]
    pub f: [Sf64; 32],

    /// Program counter.
    pub pc: u32,

    /// Floating-point CSR.
    pub fcsr: u32,

    /// Reservation slot for the atomic extension.
    ///
    /// When modifying memory outside the interpreter, this should usually be cleared.
    pub reservation: Option<u32>,
}

impl CpuState {
    /// Create a new state instance, with the given `pc` starting value.
    ///
    /// All registers are initialized to zero.
    pub fn new(pc: u32) -> Self {
        CpuState {
            x: [0; 32],
            #[cfg(feature = "rv32fd")]
            f: [Sf64(0); 32],
            pc,
            fcsr: 0,
            reservation: None,
        }
    }
}

/// Types of memory access used with the `Memory` trait.
pub enum MemoryAccess<'a, T: Copy + 'a> {
    /// Load a value from memory, placing the result in the contained reference.
    Load(&'a mut T),
    /// Store the contained value in memory.
    Store(T),
    /// Load an instruction from memory, placing it in the contained reference.
    Exec(&'a mut T),
}

/// A trait used by the interpreter to implement loads and stores.
pub trait Memory {
    /// Access the given address in memory.
    fn access<T: Copy>(&mut self, addr: u32, access: MemoryAccess<T>) -> bool;
}

/// A simple byte array can be used to implement a block of DRAM.
///
/// This is typically wrapped by a `Memory` implementation that does access control and translates
/// addresses, because by default all types of access are allowed, and the base address is 0.
impl Memory for [u8] {
    fn access<T: Copy>(&mut self, addr: u32, access: MemoryAccess<T>) -> bool {
        let addr = addr as usize;
        let end = addr + size_of::<T>();
        if let Some(slice) = self.get_mut(addr..end) {
            let ptr = slice.as_mut_ptr() as *mut T;
            match access {
                MemoryAccess::Load(dest) | MemoryAccess::Exec(dest) => {
                    unsafe { *dest = *ptr };
                },
                MemoryAccess::Store(value) => {
                    unsafe { *ptr = value };
                }
            }
            true
        } else {
            false
        }
    }
}

/// A trait used by the interpreter to implement the clock CSRs.
pub trait Clock {
    /// Read the `cycle` CSR, which counts the number of CPU cycles executed.
    fn read_cycle(&self) -> u64;

    /// Read the `time` CSR, which holds wall-clock time.
    ///
    /// The epoch and granularity are arbitrary, but the spec requires that the period is constant.
    fn read_time(&self) -> u64;

    /// Read the `instret` CSR, which counts the number of instructions executed.
    fn read_instret(&self) -> u64;

    /// Progress clocks, after the given instruction was executed.
    ///
    /// This typically increments `instret`, adds some number to `cycle`, and may also simulate
    /// `time` if it is not provided by the environment.
    ///
    /// Note that, even though these values are 64-bit, it is recommended implementations use
    /// `wrapping_add`.
    fn progress(&mut self, op: &Op);

    /// Check execution quotas. Called at the very start of `Interp::step`.
    ///
    /// When this return `false`, the virtual CPU is stopped with `CpuError::QuotaExceeded`. This
    /// allows the simulator to implement time slicing.
    ///
    /// This method is optional, and always returns `true` if not implemented.
    fn check_quota(&self) -> bool { true }
}

/// A simple implementation of the `Clock` trait.
///
/// This implementation only counts instructions. The CSRs `cycle`, `time`, and `instret` all
/// return the same counter to create a very basic simulation of time.
///
/// With the `serialize` crate feature, this structure is serializable using Serde.
#[derive(Clone,Copy,Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize,Deserialize))]
pub struct SimpleClock {
    /// Instruction counter CSR.
    pub instret: u64,
}

impl SimpleClock {
    /// Create an instance with the counter starting at 0.
    pub fn new() -> Self {
        SimpleClock { instret: 0 }
    }
}

impl Clock for SimpleClock {
    fn read_cycle(&self) -> u64 {
        self.instret
    }

    fn read_time(&self) -> u64 {
        self.instret
    }

    fn read_instret(&self) -> u64 {
        self.instret
    }

    fn progress(&mut self, _op: &Op) {
        self.instret = self.instret.wrapping_add(1);
    }
}
