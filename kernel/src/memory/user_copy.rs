use core::ptr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, PageTableFlags};

// In x86_64, canonical user-space addresses are generally below 0x0000_7FFF_FFFF_FFFF.
const USER_MAX_ADDR: u64 = 0x0000_7FFF_FFFF_FFFF;

/// Validates that a buffer is entirely within user-space.
pub fn validate_user_buffer(ptr: *const u8, size: usize) -> bool {
    let start = ptr as u64;
    let end = start.wrapping_add(size as u64);
    
    // Check for overflow and check boundary
    if end < start {
        return false;
    }
    if end > USER_MAX_ADDR {
        return false;
    }
    
    // Page Table Walk Validation
    let (cr3, _) = Cr3::read();
    let pml4_ptr = crate::memory::physical_to_virtual(cr3.start_address().as_u64()) as *const PageTable;
    let pml4 = unsafe { &*pml4_ptr };
    
    // We only check the start address for V1 simplicity. In a real OS we would check all touched pages.
    let addr = start;
    let p4_idx = (addr >> 39) & 0x1FF;
    let p3_idx = (addr >> 30) & 0x1FF;
    let p2_idx = (addr >> 21) & 0x1FF;
    let p1_idx = (addr >> 12) & 0x1FF;
    
    let p4_entry = &pml4[p4_idx as usize];
    if !p4_entry.flags().contains(PageTableFlags::PRESENT) || !p4_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) { return false; }
    
    let pdpt_ptr = crate::memory::physical_to_virtual(p4_entry.addr().as_u64()) as *const PageTable;
    let pdpt = unsafe { &*pdpt_ptr };
    let p3_entry = &pdpt[p3_idx as usize];
    if !p3_entry.flags().contains(PageTableFlags::PRESENT) || !p3_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) { return false; }
    if p3_entry.flags().contains(PageTableFlags::HUGE_PAGE) { return true; }
    
    let pd_ptr = crate::memory::physical_to_virtual(p3_entry.addr().as_u64()) as *const PageTable;
    let pd = unsafe { &*pd_ptr };
    let p2_entry = &pd[p2_idx as usize];
    if !p2_entry.flags().contains(PageTableFlags::PRESENT) || !p2_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) { return false; }
    if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) { return true; }
    
    let pt_ptr = crate::memory::physical_to_virtual(p2_entry.addr().as_u64()) as *const PageTable;
    let pt = unsafe { &*pt_ptr };
    let p1_entry = &pt[p1_idx as usize];
    if !p1_entry.flags().contains(PageTableFlags::PRESENT) || !p1_entry.flags().contains(PageTableFlags::USER_ACCESSIBLE) { return false; }
    
    true
}

/// Copies data from a user-space pointer into a kernel slice safely.
/// Returns false if the pointer is invalid.
pub fn copy_from_user(user_ptr: *const u8, kernel_buf: &mut [u8]) -> bool {
    if !validate_user_buffer(user_ptr, kernel_buf.len()) {
        return false;
    }
    
    // In a fully complete OS, this would use a safe copy macro that handles
    // SMAP (clac/stac) and catches Page Faults via an exception table.
    // For now, we manually temporarily disable SMAP to allow reading, 
    // and assume the user memory is mapped (no demand paging yet).
    unsafe {
        x86_64::instructions::interrupts::without_interrupts(|| {
            // Disable SMAP (clear AC flag in RFLAGS or clear CR4.SMAP, usually stac/clac)
            core::arch::asm!("stac");
            ptr::copy_nonoverlapping(user_ptr, kernel_buf.as_mut_ptr(), kernel_buf.len());
            core::arch::asm!("clac");
        });
    }
    
    true
}

/// Copies data from a kernel slice to a user-space pointer safely.
/// Returns false if the pointer is invalid.
pub fn copy_to_user(kernel_buf: &[u8], user_ptr: *mut u8) -> bool {
    if !validate_user_buffer(user_ptr, kernel_buf.len()) {
        return false;
    }
    
    unsafe {
        x86_64::instructions::interrupts::without_interrupts(|| {
            core::arch::asm!("stac");
            ptr::copy_nonoverlapping(kernel_buf.as_ptr(), user_ptr, kernel_buf.len());
            core::arch::asm!("clac");
        });
    }
    
    true
}
