#![allow(clippy::cast_lossless, clippy::transmute_ptr_to_ref)]

//! A simple copy-free ELF parser.
//!
//! This parser is limited, and parses only the specific kind of ELF files we expect to run.
//!
//! `Elf32::parse` can be used to parse a byte array into structs that reference the original data.
//! Note that these structs also hold values in the original endianness.

use std::mem::{size_of, transmute};
use std::slice;

/// Expected ELF magic value.
pub const ELF_IDENT_MAGIC: u32 = 0x7f45_4c46;
/// Expected ELF identity version.
pub const ELF_IDENT_VERSION_CURRENT: u8 = 1;
/// 32-bit ELF class value.
pub const ELF_IDENT_CLASS_32: u8 = 1;
/// Little-endian ELF datatype value.
pub const ELF_IDENT_DATA_2LSB: u8 = 1;
/// System V ABI type value.
pub const ELF_IDENT_ABI_SYSV: u8 = 0;
/// Executable type value.
pub const ELF_TYPE_EXECUTABLE: u16 = 2;
/// RISC-V machine type value.
pub const ELF_MACHINE_RISCV: u16 = 243;
/// Expected ELF version.
pub const ELF_VERSION_CURRENT: u32 = 1;
/// Program header bit indicating a loadable entry.
pub const ELF_PROGRAM_TYPE_LOADABLE: u32 = 1;
/// Section header type indicating space with no data (bss).
pub const ELF_SECTION_TYPE_NOBITS: u32 = 8;

trait ElfFileAddressable {
    fn get_range(&self) -> (u32, u32);
}

/// ELF identity header.
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct ElfIdent {
    /// ELF magic value, matches `ELF_IDENT_MAGIC`.
    pub magic: u32,
    /// ELF class, one of `ELF_IDENT_CLASS_*`.
    pub class: u8,
    /// Data type of the remainder of the file, one of `ELF_IDENT_DATA_*`.
    pub data: u8,
    /// Version of the header, matches `ELF_IDENT_VERSION_CURRENT`.
    pub version: u8,
    /// ABI type, one of `ELF_IDENT_ABI_*`.
    pub abi: u8,
    /// ABI version.
    pub abi_version: u8,
    /// Unused padding.
    pub padding: [u8; 7],
}

/// ELF 32-bit header.
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct ElfHeader32 {
    /// File type, one of `ELF_TYPE_*`.
    pub typ: u16,
    /// Machine type, one of `ELF_MACHINE_*`.
    pub machine: u16,
    /// ELF version, matches `ELF_VERSION_CURRENT`.
    pub version: u32,
    /// Memory address of the entry point.
    pub entry: u32,
    /// Offset in the file of the program header table.
    pub phoff: u32,
    /// Offset in the file of the section header table.
    pub shoff: u32,
    /// Architecture-specific flags.
    pub flags: u32,
    /// Size of this header.
    pub ehsize: u16,
    /// Number of program header table enties.
    pub phentsize: u16,
    /// Size of a program header.
    pub phnum: u16,
    /// Number of section header table enties.
    pub shentsize: u16,
    /// Size of a section header.
    pub shnum: u16,
    /// Section header table index of the entry containing section names.
    pub shstrndx: u16,
}

/// ELF 32-bit program header.
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct ElfProgramHeader32 {
    /// Type, a combination of `ELF_PROGRAM_TYPE_*`
    pub typ: u32,
    /// Offset in the file of the program image.
    pub offset: u32,
    /// Virtual address in memory.
    pub vaddr: u32,
    /// Optional physical address in memory.
    pub paddr: u32,
    /// Size of the image in the file.
    pub filesz: u32,
    /// Size of the image in memory.
    pub memsz: u32,
    /// Type-specific flags.
    pub flags: u32,
    /// Memory alignment in bytes.
    pub align: u32,
}
impl ElfFileAddressable for ElfProgramHeader32 {
    fn get_range(&self) -> (u32, u32) {
        (self.offset, self.filesz)
    }
}

/// ELF 32-bit section header.
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
pub struct ElfSectionHeader32 {
    /// Index in the string section containing the section name.
    pub name: u32,
    /// Section type, one of `ELF_SECTION_TYPE_*`.
    pub typ: u32,
    /// Flags, a combination of `ELF_SECTION_FLAG_*`.
    pub flags: u32,
    /// Virtual address in memory.
    pub addr: u32,
    /// Offset in the file of the setion image.
    pub offset: u32,
    /// Size of the image in the file.
    pub size: u32,
    /// Optional linked section index.
    pub link: u32,
    /// Type-specific info.
    pub info: u32,
    /// Memory alignment in bytes.
    pub addralign: u32,
    /// For sections with fixed-sized entries, the size of each entry.
    pub entsize: u32,
}
impl ElfFileAddressable for ElfSectionHeader32 {
    fn get_range(&self) -> (u32, u32) {
        (
            self.offset,
            if self.typ == ELF_SECTION_TYPE_NOBITS {
                0
            } else {
                self.size
            },
        )
    }
}

/// ELF 32-bit file structure.
#[derive(Debug)]
pub struct Elf32<'a> {
    /// The identity header.
    pub ident: &'a ElfIdent,
    /// The main header.
    pub header: &'a ElfHeader32,
    /// Program headers.
    pub ph: Vec<&'a ElfProgramHeader32>,
    /// Section headers.
    pub sh: Vec<&'a ElfSectionHeader32>,
    /// Program data.
    pub p: Vec<&'a [u8]>,
    /// Section data.
    pub s: Vec<&'a [u8]>,
}

impl<'a> Elf32<'a> {
    /// Parse an ELF file, and return structs referencing the data.
    pub fn parse(data: &'a [u8]) -> Result<Elf32<'a>, String> {
        if data.len() < size_of::<ElfIdent>() + size_of::<ElfHeader32>() {
            return Err("file too short to contain headers".to_owned());
        }

        let ident: &'a ElfIdent = unsafe { transmute(data.as_ptr()) };
        if u32::from_be(ident.magic) != ELF_IDENT_MAGIC {
            return Err("magic mismatch, likely not an ELF".to_owned());
        }
        if ident.version != ELF_IDENT_VERSION_CURRENT {
            let ident_version = ident.version;
            return Err(format!("unsupported version {}", ident_version));
        }
        if ident.class != ELF_IDENT_CLASS_32 {
            return Err("only 32-bit class supported".to_owned());
        }

        let header: &'a ElfHeader32 =
            unsafe { transmute(data.as_ptr().add(size_of::<ElfIdent>())) };
        if header.version != ELF_VERSION_CURRENT {
            let header_version = header.version;
            return Err(format!("unsupported version {}", header_version));
        }
        if header.typ != ELF_TYPE_EXECUTABLE {
            let header_typ = header.typ;
            return Err(format!("unsupported type {}", header_typ));
        }

        let (ph, p) = resolve_parts::<ElfProgramHeader32>(
            data,
            header.phoff,
            header.phentsize,
            header.phnum,
        )?;
        let (sh, s) = resolve_parts::<ElfSectionHeader32>(
            data,
            header.shoff,
            header.shentsize,
            header.shnum,
        )?;

        Ok(Elf32 {
            ident,
            header,
            ph,
            sh,
            p,
            s,
        })
    }
}

fn resolve_parts<'a, T>(
    data: &'a [u8],
    offset: u32,
    entsize16: u16,
    num16: u16,
) -> Result<(Vec<&'a T>, Vec<&'a [u8]>), String>
where
    T: ElfFileAddressable,
{
    let entsize = entsize16 as u32;
    let num = num16 as u32;

    let headers = if offset == 0 {
        Vec::new()
    } else {
        if (entsize as usize) < size_of::<T>() {
            return Err("headers smaller than defined in specification".to_owned());
        }
        if data.len() < (offset + entsize * num) as usize {
            return Err("reference to data beyond end of file".to_owned());
        }
        (0..num)
            .map(|i| unsafe { transmute(data.as_ptr().offset((offset + i * entsize) as isize)) })
            .collect::<Vec<&'a T>>()
    };

    let blocks = headers
        .iter()
        .map(|h| -> Result<&'a [u8], String> {
            let (offset, size) = h.get_range();
            if size == 0 {
                Ok(&[])
            } else if data.len() < (offset + size) as usize {
                Err("reference to data beyond end of file".to_owned())
            } else {
                Ok(unsafe {
                    slice::from_raw_parts(data.as_ptr().offset(offset as isize), size as usize)
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((headers, blocks))
}
