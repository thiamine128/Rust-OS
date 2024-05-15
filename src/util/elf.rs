use core::{cmp::min, mem::size_of};

use crate::{err::Error, memory::mmu::{VirtAddr, PAGE_SIZE, PTE_D, PTE_V}};

type Elf32Half = u16;
type Elf32Word = u32;
type Elf32Sword = i32;
type Elf32Xword = u64;
type Elf32Sxword = i64;
type Elf32Addr = u32;
type Elf32Off = u32;
type Elf32Section = u16;
type Elf32Symndx = u32;

pub const EI_NIDENT: usize = 16;
pub const EI_MAG0: usize = 0;
pub const ELFMAG0: u8 = 0x7f;
pub const EI_MAG1: usize = 1;
pub const ELFMAG1: u8 = b'E';
pub const EI_MAG2: usize = 2;
pub const ELFMAG2: u8 = b'L';
pub const EI_MAG3: usize = 3;
pub const ELFMAG3: u8 = b'F';

#[repr(C)]
pub struct Elf32Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf32Half,
    e_machine: Elf32Half,
    e_version: Elf32Word,
    pub e_entry: Elf32Addr,
    e_phoff: Elf32Off,
    e_shoff: Elf32Off,
    e_flags: Elf32Word,
    e_ehsize: Elf32Half,
    e_phentsize: Elf32Half,
    e_phnum: Elf32Half,
    e_shentsize: Elf32Half,
    e_shnum: Elf32Half,
    e_shstrndx: Elf32Half
}

#[repr(C)]
pub struct Elf32Phdr {
    pub p_type: Elf32Word,
    pub p_offset: Elf32Off,
    p_vaddr: Elf32Addr,
    p_paddr: Elf32Addr,
    p_filesz: Elf32Word,
    p_memsz: Elf32Word,
    p_flags: Elf32Word,
    p_align: Elf32Word
}

pub struct PhdrIterator<'a> {
    ehdr: &'a Elf32Ehdr,
    ind: usize
}

/* Legal values for p_type (segment type).  */

pub const PT_NULL: usize = 0;	     /* Program header table entry unused */
pub const PT_LOAD: u32 = 1;	     /* Loadable program segment */
pub const PT_DYNAMIC: usize = 2;	     /* Dynamic linking information */
pub const PT_INTERP: usize = 3;	     /* Program interpreter */
pub const PT_NOTE: usize = 4;	     /* Auxiliary information */
pub const PT_SHLIB: usize = 5;	     /* Reserved */
pub const PT_PHDR: usize = 6;	     /* Entry for header table itself */
pub const PT_NUM: usize = 7;	     /* Number of defined types.  */
pub const PT_LOOS: usize = 0x60000000;   /* Start of OS-specific */
pub const PT_HIOS: usize = 0x6fffffff;   /* End of OS-specific */
pub const PT_LOPROC: usize = 0x70000000; /* Start of processor-specific */
pub const PT_HIPROC: usize = 0x7fffffff; /* End of processor-specific */

/* Legal values for p_flags (segment flags).  */

pub const PF_X: u32 = 1 << 0;	       /* Segment is executable */
pub const PF_W: u32 = 1 << 1;	       /* Segment is writable */
pub const PF_R: u32 = 1 << 2;	       /* Segment is readable */
pub const PF_MASKPROC: u32 = 0xf0000000; /* Processor-specific */

/* Utils provided by our ELF loader. */


pub fn elf_from(binary: &[u8], size: usize) -> Option<&Elf32Ehdr> {
    let ehdr_ptr = binary.as_ptr() as *const Elf32Ehdr;
    let ehdr = unsafe { ehdr_ptr.as_ref() };
    let ehdr = ehdr.unwrap();
    if size >= size_of::<Elf32Ehdr>() && ehdr.e_ident[EI_MAG0] == ELFMAG0
        && ehdr.e_ident[EI_MAG1] == ELFMAG1 && ehdr.e_ident[EI_MAG2] == ELFMAG2
        && ehdr.e_ident[EI_MAG3] == ELFMAG3 {
        Some(ehdr)
    } else {
        None
    }
}

pub fn elf_load_seg<F>(ph: &Elf32Phdr, bin: &[u8], mut map_page: F) -> Result<(), Error>
where F: FnMut(VirtAddr, usize, usize, Option<&[u8]>, usize)->Result<(), Error> {
    let va = VirtAddr::new(ph.p_vaddr as usize);
    let bin_size = ph.p_filesz as usize;
    let sgsize = ph.p_memsz as usize;
    let mut perm = PTE_V;

    if ph.p_flags & PF_W != 0 {
        perm |= PTE_D;
    }

    let offset = va.as_usize() - va.page_align_down().as_usize();
    if offset != 0 {
        map_page(va, offset, perm, Some(bin), min(bin_size, PAGE_SIZE - offset))?;
    }
    let st = if offset != 0 {
        min(bin_size, PAGE_SIZE - offset)
    } else {
        0
    };
    let mut i = st;
    while i < bin_size {
        map_page(va + i, 0, perm, Some(&bin[i..]), min(bin_size - i, PAGE_SIZE))?;
        i += PAGE_SIZE;
    }
    while i < sgsize {
        map_page(va + i, 0, perm, None, min(sgsize - i, PAGE_SIZE))?;
        i += PAGE_SIZE;
    }
    Ok(())
}

impl<'a> Iterator for PhdrIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ind < self.ehdr.e_phnum as usize {
            let result = self.ehdr.e_phoff as usize + self.ind * self.ehdr.e_phentsize as usize;
            self.ind += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl Elf32Ehdr {
    pub fn phdr_iter(&self) -> PhdrIterator {
        PhdrIterator {
            ehdr: self,
            ind: 0
        }
    }
}