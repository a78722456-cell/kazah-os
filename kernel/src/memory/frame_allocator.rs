use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use limine::memmap::{Entry, MEMMAP_USABLE};

/// A true Bitmap Physical Memory Allocator.
pub struct BitmapFrameAllocator {
    bitmap: &'static mut [u64],
    last_allocated_index: usize,
}

impl BitmapFrameAllocator {
    pub unsafe fn new(memory_map: &'static [&'static Entry], hhdm_offset: u64) -> Self {
        // 1. Find max physical address
        let max_addr = memory_map.iter()
            .map(|r| r.base + r.length)
            .max()
            .unwrap_or(0);
        
        let total_frames = max_addr / 4096;
        let bitmap_u64_count = ((total_frames + 63) / 64) as usize;
        let bitmap_bytes = (bitmap_u64_count * 8) as u64;
        
        // 2. Find a usable region large enough for the bitmap
        let mut bitmap_ptr: *mut u64 = core::ptr::null_mut();
        let mut bitmap_phys_base = 0;
        
        for region in memory_map.iter() {
            if region.type_ == MEMMAP_USABLE && region.length >= bitmap_bytes {
                bitmap_phys_base = region.base;
                bitmap_ptr = (bitmap_phys_base + hhdm_offset) as *mut u64;
                break;
            }
        }
        
        if bitmap_ptr.is_null() {
            panic!("Not enough memory for Bitmap Frame Allocator!");
        }
        
        // 3. Initialize bitmap: 1 means allocated/reserved, 0 means free.
        // Initially set all to 1 (reserved)
        let bitmap = core::slice::from_raw_parts_mut(bitmap_ptr, bitmap_u64_count);
        for i in 0..bitmap.len() {
            bitmap[i] = !0;
        }
        
        let mut allocator = Self {
            bitmap,
            last_allocated_index: 0,
        };
        
        // 4. Free the usable regions
        for region in memory_map.iter() {
            if region.type_ == MEMMAP_USABLE {
                allocator.free_region(region.base, region.length);
            }
        }
        
        // 5. Re-reserve the bitmap's own memory
        allocator.reserve_region(bitmap_phys_base, bitmap_bytes);
        
        allocator
    }
    
    pub fn reserve_region(&mut self, base: u64, length: u64) {
        let start_frame = base / 4096;
        let end_frame = (base + length + 4095) / 4096;
        for i in start_frame..end_frame {
            self.set_bit(i as usize);
        }
    }
    
    pub fn free_region(&mut self, base: u64, length: u64) {
        // Only free fully aligned frames
        let start_frame = (base + 4095) / 4096;
        let end_frame = (base + length) / 4096;
        for i in start_frame..end_frame {
            self.clear_bit(i as usize);
        }
    }
    
    fn set_bit(&mut self, frame: usize) {
        let idx = frame / 64;
        let bit = frame % 64;
        if idx < self.bitmap.len() {
            self.bitmap[idx] |= 1 << bit;
        }
    }
    
    fn clear_bit(&mut self, frame: usize) {
        let idx = frame / 64;
        let bit = frame % 64;
        if idx < self.bitmap.len() {
            self.bitmap[idx] &= !(1 << bit);
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let len = self.bitmap.len();
        
        // Search starting from last_allocated_index
        for i in 0..len {
            let idx = (self.last_allocated_index + i) % len;
            if self.bitmap[idx] != !0 { // There is at least one 0 bit
                for bit in 0..64 {
                    if (self.bitmap[idx] & (1 << bit)) == 0 {
                        self.bitmap[idx] |= 1 << bit;
                        self.last_allocated_index = idx;
                        let frame_idx = idx * 64 + bit;
                        let addr = PhysAddr::new((frame_idx as u64) * 4096);
                        return Some(PhysFrame::containing_address(addr));
                    }
                }
            }
        }
        
        None // Out of memory
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
        let frame_idx = frame.start_address().as_u64() / 4096;
        self.clear_bit(frame_idx as usize);
    }
}
