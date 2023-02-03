extern crate rayon;
extern crate rvsim;

use rayon::prelude::*;
use rvsim::*;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{exit, Command};

include!("env/meta.rs");

struct TestMemory {
    dram: Vec<u8>,
}

impl TestMemory {
    const DRAM_BASE: u32 = 0x1000_0000;
    const DRAM_SIZE: usize = 0x10_0000;

    fn new() -> Self {
        Self {
            dram: vec![0; Self::DRAM_SIZE],
        }
    }
}

impl Memory for TestMemory {
    fn access<T: Copy>(&mut self, addr: u32, access: MemoryAccess<T>) -> bool {
        if addr >= Self::DRAM_BASE {
            Memory::access(&mut self.dram[..], addr - Self::DRAM_BASE, access)
        } else {
            false
        }
    }
}

#[test]
fn riscv_tests() {
    build_riscv_tests();
    run_riscv_tests();
}

fn build_riscv_tests() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let isa_tests_dir = PathBuf::from("vendor/riscv-tests/isa");

    ISA_TESTS.par_iter().for_each(|&(set, name)| {
        let out_file = out_path.join(format!("test-{}-{}", set, name));
        if out_file.exists() {
            return;
        }

        let in_file = format!("{}/{}.S", set, name);
        let mut cmd = Command::new("riscv32-none-elf-gcc");
        cmd.args([
            "-static",
            "-march=rv32g",
            "-mabi=ilp32",
            "-nostdlib",
            "-nostartfiles",
            "-I./../../../tests/env",
            "-I./macros/scalar",
            "-T./../../../tests/env/link.ld",
            "-Wl,--no-warn-rwx-segments",
            "-o",
            &out_file.to_string_lossy(),
            &in_file,
        ]);
        cmd.current_dir(&isa_tests_dir);

        println!("+ {:?}", cmd);
        if !cmd.status().unwrap().success() {
            exit(1);
        }
    });
}

fn run_riscv_tests() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let results = ISA_TESTS
        .par_iter()
        .map(|&(set, name)| {
            let bin = format!("test-{}-{}", set, name);
            let bin_path = out_path.join(&bin);
            (bin, run_riscv_test(&bin_path.to_string_lossy()))
        })
        .collect::<Vec<_>>();

    let mut ok = true;
    for (bin, result) in results {
        match result {
            Ok(_) => {}
            Err(msg) => {
                println!("FAIL: {} - {}", bin, msg);
                ok = false;
            }
        }
    }

    if !ok {
        panic!("Some CPU tests failed");
    }
}

fn run_riscv_test(filename: &str) -> Result<(), String> {
    let mut data = Vec::new();
    File::open(filename)
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();

    let elf = elf::Elf32::parse(&data).unwrap();
    if elf.ident.data != elf::ELF_IDENT_DATA_2LSB
        || elf.ident.abi != elf::ELF_IDENT_ABI_SYSV
        || elf.header.typ != elf::ELF_TYPE_EXECUTABLE
        || elf.header.machine != elf::ELF_MACHINE_RISCV
    {
        return Err("Unsupported executable format".to_string());
    }

    let mut mem = TestMemory::new();
    for (i, ph) in elf.ph.iter().enumerate() {
        if ph.typ == elf::ELF_PROGRAM_TYPE_LOADABLE {
            let offset = (ph.vaddr - TestMemory::DRAM_BASE) as usize;
            let mut dest = &mut mem.dram[offset..];
            dest.write_all(elf.p[i])
                .map_err(|e| format!("Failed to load executable image: {}", e))?;
        }
    }

    let mut state = CpuState::new(elf.header.entry);
    let mut clock = SimpleClock::new();
    match Interp::new(&mut state, &mut mem, &mut clock).run() {
        (CpuError::Ecall, _) => {
            if state.x[3] != 1 {
                return Err(format!("FAIL {}", state.x[3] >> 1));
            }
        }
        (err, _) => {
            return Err(format!("EXIT {:?}", err));
        }
    }

    Ok(())
}
