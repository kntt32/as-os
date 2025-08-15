#![no_std]

pub mod types;

use core::iter::Iterator;
use core::mem;
use core::mem::size_of;
use core::slice::SliceIndex;
use types::*;

pub struct Elf64<'a> {
    bin: &'a [u8],
}

impl<'a> Elf64<'a> {
    pub fn new(bin: &'a [u8]) -> Result<Self, &'static str> {
        let elf64 = Self { bin: bin };

        if elf64.is_valid() {
            Ok(elf64)
        } else {
            Err("invalid elf64 file")
        }
    }

    fn is_valid(&self) -> bool {
        let Ok(elf_header) = self.elf_header() else {
            return false;
        };

        elf_header.e_ident[0] == Elf64Ehdr::ELFMAG0
            && elf_header.e_ident[1] == Elf64Ehdr::ELFMAG1
            && elf_header.e_ident[2] == Elf64Ehdr::ELFMAG2
            && elf_header.e_ident[3] == Elf64Ehdr::ELFMAG3
    }

    pub fn entry(&self) -> Result<u64, &'static str> {
        Ok(self.elf_header()?.e_entry)
    }

    pub fn elf_header(&self) -> Result<Elf64Ehdr, &'static str> {
        if size_of::<Elf64Ehdr>() <= self.bin.len() {
            let header_ptr: *const u8 = self.bin.as_ptr();
            let mut elf_header: Elf64Ehdr = unsafe { mem::zeroed() };
            unsafe {
                header_ptr.copy_to(&mut elf_header as *mut _ as *mut u8, size_of::<Elf64Ehdr>())
            };
            Ok(elf_header)
        } else {
            Err("failed to get elf header")
        }
    }

    pub fn program_headers(&self) -> Result<Elf64PhdrIter<'_>, &'static str> {
        let elf_header = self.elf_header()?;

        let phdr_offset = elf_header.e_phoff as usize;
        let phdr_entsize = elf_header.e_phentsize as usize;
        let phdr_num = elf_header.e_phnum as usize;

        let elf64_phdrs = self.get(phdr_offset .. phdr_offset + phdr_entsize * phdr_num)?;

        Elf64PhdrIter::new(elf64_phdrs, phdr_num, phdr_entsize)
    }

    pub fn get<I: SliceIndex<[u8]>>(&self, range: I) -> Result<&<I as SliceIndex<[u8]>>::Output, &'static str> {
        if let Some(slice) = self.bin.get(range) {
            Ok(slice)
        }else {
            Err("out of range")
        }
    }

    pub fn expand_info(&self) -> Result<Elf64ExpandInfo, &'static str> {
        let mut flag = true;
        let mut lower_addr = 0;
        let mut upper_addr = 0;

        for phdr in self.program_headers()? {
            match phdr.p_type {
                Elf64Phdr::PT_NULL => (),
                Elf64Phdr::PT_LOAD => {
                    if phdr.p_memsz < phdr.p_filesz
                        || (self.bin.len() as u64)
                            < phdr.p_offset + phdr.p_filesz
                    {
                        return Err("program header is corrupted");
                    }

                    if flag {
                        lower_addr = phdr.p_vaddr;
                        upper_addr = phdr.p_vaddr + phdr.p_memsz;
                        flag = false;
                        continue;
                    }

                    if phdr.p_vaddr < lower_addr {
                        lower_addr = phdr.p_vaddr;
                    }
                    if upper_addr < phdr.p_vaddr + phdr.p_memsz {
                        upper_addr = phdr.p_vaddr + phdr.p_memsz;
                    }
                }
                _ => (),
            }
        }

        if !flag {
            assert!(lower_addr <= upper_addr);
            Ok(Elf64ExpandInfo {
                lower_addr: lower_addr,
                upper_addr: upper_addr,
            })
        } else {
            Err("no any program headers were found")
        }
    }

    pub fn expand(&self, buff: &mut [u8]) -> Result<(), &'static str> {
        let expand_info = self.expand_info()?;
        let expand_size = expand_info.upper_addr - expand_info.lower_addr;

        if (buff.len() as u64) < expand_size {
            return Err("too small buffer size");
        }

        buff.fill(0x00);

        let expand_base = expand_info.lower_addr;
        let program_headers = self.program_headers()?;
        for phdr in program_headers {
            match phdr.p_type {
                Elf64Phdr::PT_NULL => (),
                Elf64Phdr::PT_LOAD => {
                    let file_offset = phdr.p_offset as usize;
                    let file_offset_top = (phdr.p_offset + phdr.p_filesz) as usize;
                    let load_src: &[u8] =
                        &self.bin[file_offset .. file_offset_top];
                    let dst_offset = (phdr.p_vaddr - expand_base) as usize;
                    let dst_offset_top = (phdr.p_vaddr + phdr.p_filesz - expand_base) as usize;
                    buff[dst_offset .. dst_offset_top].copy_from_slice(load_src);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Elf64ExpandInfo {
    pub lower_addr: u64,
    pub upper_addr: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Elf64PhdrIter<'a> {
    bin: &'a [u8],
    ph_num: usize,
    ph_entsize: usize,
}

impl<'a> Elf64PhdrIter<'a> {
    pub fn new(bin: &'a [u8], ph_num: usize, ph_entsize: usize) -> Result<Self, &'static str> {
        if ph_num * ph_entsize == bin.len() {
            Ok(Self {
                bin: bin,
                ph_num: ph_num,
                ph_entsize: ph_entsize,
            })
        } else {
            Err("invalid program headers found")
        }
    }
}

impl<'a> Iterator for Elf64PhdrIter<'a> {
    type Item = Elf64Phdr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ph_num != 0 {
            assert!(self.ph_entsize <= self.bin.len());
            let ptr: *const u8 = self.bin.as_ptr();
            let mut phdr: Elf64Phdr = unsafe { mem::zeroed() };
            unsafe {
                ptr.copy_to(&mut phdr as *mut _ as *mut u8, size_of::<Elf64Phdr>());
            }
            self.bin = &self.bin[self.ph_entsize..];
            self.ph_num -= 1;

            Some(phdr)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Ehdr {
    e_ident: [u8; Self::EI_NIDENT],
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    e_entry: Elf64Addr,
    e_phoff: Elf64Off,
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}

impl Elf64Ehdr {
    pub const EI_NIDENT: usize = 16;

    pub const ELFMAG0: u8 = 0x7f;
    pub const ELFMAG1: u8 = b'E';
    pub const ELFMAG2: u8 = b'L';
    pub const ELFMAG3: u8 = b'F';

    pub const ELFCLASS64: u8 = 2;

    pub const ELFDATA2LSB: u8 = 1;

    pub const ELFVERSION: u8 = 1;

    pub const ELFOSABI_LINUX: u8 = 3;

    pub const ET_EXEC: Elf64Half = 2;
    pub const EM_X86_64: Elf64Half = 62;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Phdr {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_filesz: Elf64Xword,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

impl Elf64Phdr {
    pub const PT_NULL: Elf64Word = 0;
    pub const PT_LOAD: Elf64Word = 1;

    pub fn x_flag(&self) -> bool {
        self.p_flags & 0x1 != 0
    }

    pub fn w_flag(&self) -> bool {
        self.p_flags & 0x2 != 0
    }

    pub fn r_flag(&self) -> bool {
        self.p_flags & 0x4 != 0
    }
}
