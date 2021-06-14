use crate::bootboot_bindings::*;

/*
#define MMapEnt_Ptr(a)  (a->ptr)
#define MMapEnt_Size(a) (a->size & 0xFFFFFFFFFFFFFFF0)
#define MMapEnt_Type(a) (a->size & 0xF)
#define MMapEnt_IsFree(a) ((a->size&0xF)==1)
#define MMAP_USED     0   /* don't use. Reserved or unknown regions */
#define MMAP_FREE     1   /* usable memory */
#define MMAP_ACPI     2   /* acpi memory, volatile and non-volatile as well */
#define MMAP_MMIO     3   /* memory mapped IO region */
*/
impl MemoryMapInfo {
    pub fn ptr(&self) -> usize {
        self.ptr as usize
    }

    /// Size of memory area in bytes.
    pub fn size(&self) -> usize {
        (self.size & 0xFFFFFFFFFFFFFFF0) as usize
    }

    /// Returns true if the area can be used by OS.
    pub fn is_free(&self) -> bool {
        let is_free = (self.size & 0xF) == 1;
        is_free
    }

    /// Get the type of memory entry.
    pub fn get_type(&self) -> MemoryMapEntryType {
        let _ptr = self.ptr as *mut u8;
        let _size = self.size as *mut u8;
        match self.size & 0xF {
            0 => MemoryMapEntryType::Used,
            1 => MemoryMapEntryType::Free,
            2 => MemoryMapEntryType::Acpi,
            3 => MemoryMapEntryType::Mmio,
            _ => MemoryMapEntryType::Used,
        }
    }

    pub fn end_address(&self) -> u64 {
        self.ptr + self.size() as u64
    }
}

impl BOOTBOOT {
    pub fn get_mmap_entries(&self) -> &'static [MemoryMapInfo] {
        let num_mmap_entries = (self.size - 128) / 16;
        let addr = core::ptr::addr_of!(self.mmap.ptr);
        unsafe {
            core::slice::from_raw_parts(addr as *const MemoryMapInfo, num_mmap_entries as usize)
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryMapEntryType {
    Used,
    Free,
    Acpi,
    Mmio,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryMapInfo {
    ptr: u64,
    size: u64,
}
