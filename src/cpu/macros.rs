/// Finish an instruction, progressing the PC.
macro_rules! end_op {
    ( $interp:expr ) => ({
        $interp.state.pc += $interp.instsz;
        return Ok(());
    });
    ( $interp:expr , $name:ident ) => ({
        $interp.state.pc += $interp.instsz;
        return Err(CpuError::$name);
    });
}

/// Finish a jump instruction, performing an absolute jump.
macro_rules! end_jump_op {
    ( $interp:expr , $pc:expr ) => ({
        let pc = $pc;
        #[cfg(feature = "rv32c")]
        {
            if pc % 2 != 0 {
                return Err(CpuError::MisalignedFetch);
            }
        }
        #[cfg(not(feature = "rv32c"))]
        {
            if pc % 4 != 0 {
                return Err(CpuError::MisalignedFetch);
            }
        }

        $interp.state.pc = pc;
        return Ok(());
    });
}

/// Finish a branch instruction, performing a relative jump.
macro_rules! end_branch_op {
    ( $interp:expr , $imm:expr ) => ({
        let pc = $interp.state.pc.wrapping_add($imm as u32);
        #[cfg(feature = "rv32c")]
        {
            if pc % 2 != 0 {
                return Err(CpuError::MisalignedFetch);
            }
        }
        #[cfg(not(feature = "rv32c"))]
        {
            if pc % 4 != 0 {
                return Err(CpuError::MisalignedFetch);
            }
        }

        $interp.state.pc = pc;
        return Ok(());
    });
}

/// Wrap a block, writing the result to integer register `$rd`.
/// The block is not executed if `$rd` is 0.
macro_rules! write_rd {
    ( $interp:expr , $rd:expr , $code:block ) => ({
        if $rd != 0 {
            $interp.state.x[$rd] = $code;
        }
    })
}

/// Macro used to implement AMO instructions.
macro_rules! amo {
    ( $interp:expr , $rd:expr , $rs1:expr , $code:block ) => ({
        let addr = $interp.state.x[$rs1];
        if addr % 4 != 0 {
            end_op!($interp, MisalignedAccess);
        }

        let mut value: u32 = 0;
        if !$interp.mem.access(addr, MemoryAccess::Load(&mut value)) {
            end_op!($interp, IllegalAccess);
        }

        write_rd!($interp, $rd, { value });

        let value: u32 = $code;
        if !$interp.mem.access(addr, MemoryAccess::Store(value)) {
            end_op!($interp, IllegalAccess);
        }

        end_op!($interp);
    });
}

/// Wrap a block with a prepared softfloat environment.
/// Handles exception flags and rounding mode.
#[cfg(feature = "rv32fd")]
macro_rules! sf_wrap {
    ( $interp:expr , $rm:expr , $code:block ) => ({
        unsafe {
            sf::set_flags(0);
            sf::set_rounding_mode(match $rm {
                // Reserved values.
                5 | 6 => end_op!($interp, IllegalInstruction),
                // Dynamic rounding mode.
                7 => ($interp.state.fcsr & 0b1110_0000) >> 5,
                // Inline rounding mode. Values match with SoftFloat.
                _ => $rm,
            } as u8);
        }

        let value = $code;

        // Exception flags match with SoftFloat.
        $interp.state.fcsr |= unsafe {
            (sf::get_flags() & 0b1_1111) as u32
        };

        value
    });
    ( $interp:expr , $code:block ) => ({
        sf_wrap!($interp, 7, $code)
    });
}

/// Perform a softfloat calculation, writing the result to `$rd`.
/// Uses `sf_wrap` to prepare the environment.
#[cfg(feature = "rv32fd")]
macro_rules! sf_calc {
    ( $interp:expr , $rd:expr , $code:block ) => ({
        $interp.state.f[$rd] = sf_wrap!($interp, $code);
        end_op!($interp)
    });
    ( $interp:expr , $rm:expr , $rd:expr , $code:block ) => ({
        $interp.state.f[$rd] = sf_wrap!($interp, $rm, $code);
        end_op!($interp)
    });
}
