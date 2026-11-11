use alloc::vec::Vec;
use core::str;

pub static mut INITRD_DATA: Option<&'static [u8]> = None;

pub struct TarArchive<'a> {
    data: &'a [u8],
}

impl<'a> TarArchive<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn find(&self, filename: &str) -> Option<&'a [u8]> {
        let mut offset = 0;
        
        while offset + 512 <= self.data.len() {
            let header = &self.data[offset..offset + 512];
            
            // A block of 512 null bytes indicates the end of the archive
            if header[0] == 0 {
                break;
            }
            
            let name_bytes = &header[0..100];
            let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(100);
            let name = core::str::from_utf8(&name_bytes[..name_len]).unwrap_or("");
            
            let size_bytes = &header[124..135]; // Octal size
            let size_str = core::str::from_utf8(size_bytes).unwrap_or("").trim();
            let size = usize::from_str_radix(size_str, 8).unwrap_or(0);
            
            let content_offset = offset + 512;
            let next_offset = content_offset + (size + 511) / 512 * 512;
            
            if name == filename {
                if content_offset + size <= self.data.len() {
                    return Some(&self.data[content_offset..content_offset + size]);
                } else {
                    return None; // Archive is truncated
                }
            }
            
            offset = next_offset;
        }
        
        None
    }
}
