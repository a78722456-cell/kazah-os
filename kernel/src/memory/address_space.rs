use x86_64::structures::paging::{PageTable, PhysFrame, FrameAllocator, FrameDeallocator};
use x86_64::PhysAddr;
use crate::memory::FRAME_ALLOCATOR;
use x86_64::registers::control::Cr3;

pub struct AddressSpace {
    pub cr3: PhysAddr,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let pml4_frame = crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().allocate_frame()?;
        let pml4_addr = pml4_frame.start_address();
        
        let pml4_ptr = crate::memory::physical_to_virtual(pml4_addr.as_u64()) as *mut PageTable;
        
        unsafe {
            core::ptr::write_bytes(pml4_ptr as *mut u8, 0, 4096);
            let new_pml4 = &mut *pml4_ptr;
            
            let (current_cr3, _) = Cr3::read();
            let current_pml4_ptr = crate::memory::physical_to_virtual(current_cr3.start_address().as_u64()) as *const PageTable;
            let current_pml4 = &*current_pml4_ptr;
            
            for i in 256..512 {
                new_pml4[i] = current_pml4[i].clone();
            }
        }
        
        Some(Self { cr3: pml4_addr })
    }
    
    pub unsafe fn switch(&self) {
        let (current_cr3, flags) = Cr3::read();
        if current_cr3.start_address() != self.cr3 {
            let frame = PhysFrame::from_start_address(self.cr3).unwrap();
            Cr3::write(frame, flags);
        }
    }
    
    pub fn get_mapper(&mut self) -> x86_64::structures::paging::OffsetPageTable<'static> {
        let pml4_ptr = crate::memory::physical_to_virtual(self.cr3.as_u64()) as *mut PageTable;
        let pml4 = unsafe { &mut *pml4_ptr };
        let phys_offset = x86_64::VirtAddr::new(crate::memory::physical_to_virtual(0));
        unsafe { x86_64::structures::paging::OffsetPageTable::new(pml4, phys_offset) }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let frame = PhysFrame::from_start_address(self.cr3).unwrap();
        let pml4_ptr = crate::memory::physical_to_virtual(self.cr3.as_u64()) as *mut PageTable;
        let pml4 = unsafe { &mut *pml4_ptr };
        
        for i in 0..256 {
            let pml4_entry = &pml4[i];
            if !pml4_entry.is_unused() && pml4_entry.flags().contains(x86_64::structures::paging::PageTableFlags::PRESENT) {
                let pdp_addr = pml4_entry.addr();
                let pdp_ptr = crate::memory::physical_to_virtual(pdp_addr.as_u64()) as *mut PageTable;
                let pdp = unsafe { &mut *pdp_ptr };
                
                for j in 0..512 {
                    let pdp_entry = &pdp[j];
                    if !pdp_entry.is_unused() && pdp_entry.flags().contains(x86_64::structures::paging::PageTableFlags::PRESENT) {
                        if pdp_entry.flags().contains(x86_64::structures::paging::PageTableFlags::HUGE_PAGE) { continue; }
                        
                        let pd_addr = pdp_entry.addr();
                        let pd_ptr = crate::memory::physical_to_virtual(pd_addr.as_u64()) as *mut PageTable;
                        let pd = unsafe { &mut *pd_ptr };
                        
                        for k in 0..512 {
                            let pd_entry = &pd[k];
                            if !pd_entry.is_unused() && pd_entry.flags().contains(x86_64::structures::paging::PageTableFlags::PRESENT) {
                                if pd_entry.flags().contains(x86_64::structures::paging::PageTableFlags::HUGE_PAGE) { continue; }
                                
                                let pt_addr = pd_entry.addr();
                                let pt_ptr = crate::memory::physical_to_virtual(pt_addr.as_u64()) as *mut PageTable;
                                let pt = unsafe { &mut *pt_ptr };
                                
                                for l in 0..512 {
                                    let pt_entry = &pt[l];
                                    if !pt_entry.is_unused() && pt_entry.flags().contains(x86_64::structures::paging::PageTableFlags::PRESENT) {
                                        let page_frame = PhysFrame::from_start_address(pt_entry.addr()).unwrap();
                                        unsafe { crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().deallocate_frame(page_frame); }
                                    }
                                }
                                let pt_frame = PhysFrame::from_start_address(pt_addr).unwrap();
                                unsafe { crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().deallocate_frame(pt_frame); }
                            }
                        }
                        let pd_frame = PhysFrame::from_start_address(pd_addr).unwrap();
                        unsafe { crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().deallocate_frame(pd_frame); }
                    }
                }
                let pdp_frame = PhysFrame::from_start_address(pdp_addr).unwrap();
                unsafe { crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().deallocate_frame(pdp_frame); }
            }
        }

        unsafe {
            crate::memory::FRAME_ALLOCATOR.lock().as_mut().unwrap().deallocate_frame(frame);
        }
    }
}
