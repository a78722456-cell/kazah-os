use alloc::vec::Vec;
use crate::object::process::ProcessObject;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB, Mapper, FrameAllocator, PhysFrame};
use x86_64::VirtAddr;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

#[repr(C, packed)]
struct Elf64_Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C, packed)]
struct Elf64_Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

pub struct LoadedImage {
    pub entry_point: u64,
    pub base_address: u64,
    pub end_address: u64,
}

pub fn load_elf(data: &[u8], process: &mut ProcessObject) -> Option<LoadedImage> {
    if data.len() < core::mem::size_of::<Elf64_Ehdr>() {
        return None;
    }

    let ehdr = unsafe { &*(data.as_ptr() as *const Elf64_Ehdr) };
    
    if ehdr.e_ident[0..4] != ELF_MAGIC {
        return None;
    }

    let phoff = ehdr.e_phoff as usize;
    let phnum = ehdr.e_phnum as usize;
    let phentsize = ehdr.e_phentsize as usize;

    if data.len() < phoff + phnum * phentsize {
        return None;
    }

    let mut base_address = u64::MAX;
    let mut end_address = 0;

    // First pass to find limits
    for i in 0..phnum {
        let offset = phoff + i * phentsize;
        let phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64_Phdr) };
        if phdr.p_type == PT_LOAD {
            if phdr.p_vaddr < base_address { base_address = phdr.p_vaddr; }
            let seg_end = phdr.p_vaddr + phdr.p_memsz;
            if seg_end > end_address { end_address = seg_end; }
        }
    }

    // Now map and copy
    let mut mapper = process.address_space.get_mapper();
    let mut frame_allocator_guard = crate::memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator_guard.as_mut().unwrap();

    for i in 0..phnum {
        let offset = phoff + i * phentsize;
        let phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64_Phdr) };
        
        if phdr.p_type == PT_LOAD {
            let vaddr = phdr.p_vaddr;
            let memsz = phdr.p_memsz;
            let filesz = phdr.p_filesz;
            let file_offset = phdr.p_offset as usize;
            
            let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
            if phdr.p_flags & PF_W != 0 { flags |= PageTableFlags::WRITABLE; }
            if phdr.p_flags & PF_X == 0 { flags |= PageTableFlags::NO_EXECUTE; }

            let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr));
            let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr + memsz - 1));

            for page in Page::range_inclusive(start_page, end_page) {
                // Ignore if already mapped
                let frame = frame_allocator.allocate_frame().unwrap();
                unsafe {
                    let _ = mapper.map_to(page, frame, flags, frame_allocator).unwrap().flush();
                }
            }

            // Switch CR3 to write data securely.
            let (old_cr3, cr3_flags) = x86_64::registers::control::Cr3::read();
            unsafe { process.address_space.switch(); }

            unsafe {
                core::ptr::write_bytes(vaddr as *mut u8, 0, memsz as usize);
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(file_offset),
                    vaddr as *mut u8,
                    filesz as usize,
                );
            }

            unsafe {
                let frame = x86_64::structures::paging::PhysFrame::from_start_address(old_cr3.start_address()).unwrap();
                x86_64::registers::control::Cr3::write(frame, cr3_flags);
            }
        }
    }

    Some(LoadedImage {
        entry_point: ehdr.e_entry,
        base_address,
        end_address,
    })
}
