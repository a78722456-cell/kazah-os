pub mod address_space;
pub mod frame_allocator;
pub mod heap;
pub mod paging;
pub mod user_copy;
pub mod tar;
pub mod elf;

use core::sync::atomic::{AtomicU64, Ordering};
use limine::request::MemmapResponse;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Option<frame_allocator::BitmapFrameAllocator>> = Mutex::new(None);
}

static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);

pub fn physical_to_virtual(physical_addr: u64) -> u64 {
    physical_addr + HHDM_OFFSET.load(Ordering::Relaxed)
}

pub fn init(memmap: &'static MemmapResponse, hhdm_offset: u64) {
    HHDM_OFFSET.store(hhdm_offset, Ordering::SeqCst);
    
    // 1. Initialize physical frame allocator
    let mut frame_allocator = unsafe { frame_allocator::BitmapFrameAllocator::new(memmap.entries(), hhdm_offset) };
    
    // 2. Initialize paging
    let phys_mem_offset = VirtAddr::new(hhdm_offset);
    let mut mapper = unsafe { paging::init(phys_mem_offset) };
    
    // 3. Initialize heap allocator
    heap::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
    
    *FRAME_ALLOCATOR.lock() = Some(frame_allocator);
}
