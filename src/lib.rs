#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

use bit_field::BitField;
#[allow(unused_imports)]
use chrono::{DateTime, FixedOffset, Utc};

#[allow(clippy::clippy::redundant_static_lifetimes)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(unused)]
mod bootboot_bindings;

mod mmap;

#[cfg(feature = "alloc")]
extern crate alloc;

/**
Struct to access bootloader information.
*/
#[derive(Debug, Clone, Copy)]
pub struct BootBoot;

impl BootBoot {
    pub fn protocol() -> Protocol {
        match Self::get_bootboot().protocol.get_bits(0..1) as u32 {
            bootboot_bindings::PROTOCOL_MINIMAL => Protocol::Minimal,
            bootboot_bindings::PROTOCOL_STATIC => Protocol::Static,
            bootboot_bindings::PROTOCOL_DYNAMIC => Protocol::Dynamic,
            _ => Protocol::Unknown,
        }
    }

    pub fn endianness() -> Endian {
        match Self::get_bootboot().protocol.get_bit(7) {
            false => Endian::Little,
            true => Endian::Big,
        }
    }

    pub fn loader_type() -> Loader {
        match Self::get_bootboot().protocol.get_bits(2..6) as u32 {
            bootboot_bindings::LOADER_BIOS => Loader::Bios,
            bootboot_bindings::LOADER_UEFI => Loader::Uefi,
            bootboot_bindings::LOADER_RPI => Loader::Rpi,
            bootboot_bindings::LOADER_COREBOOT => Loader::CoreBoot,
            _ => Loader::Unknown,
        }
    }

    /**
    The number of CPU cores. On Symmetric Multi Processing platforms this can be larger than 1.
    */
    pub fn num_cores() -> usize {
        Self::get_bootboot().numcores as _
    }

    /**
    The BootStrap Processor ID on platforms that support SMP (Local APIC ID on x86_64).
    */
    pub fn bsp_id() -> usize {
        Self::get_bootboot().bspid as _
    }

    /**
    The machine’s detected timezone if such a thing is supported on the platform.
    */
    pub fn timezone_offset() -> FixedOffset {
        FixedOffset::east(Self::get_bootboot().timezone as i32 * 60)
    }

    /**
    The UTC date of boot in binary coded decimal on platforms that have RTC chip. The first two bytes in
    hexadecimal gives the year, for example 0x20 0x17, then one byte the month 0x12, one byte the day
    0x31. Followed by hours 0x23, minutes 0x59 and seconds 0x59 bytes. The last byte can store 1/100th
    second precision, but in lack of support on most platforms, it is 0x00. Not influenced by the timezone
    field
    */
    pub fn datetime_raw() -> [u8; 8] {
        Self::get_bootboot().datetime
    }

    /**
    The UTC date of boot in binary coded decimal on platforms that have RTC chip.
    Not influenced by the timezone.
    */
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn datetime() -> DateTime<Utc> {
        let a = Self::get_bootboot().datetime;
        let b = alloc::format!(
            "{:02x}{:02x} {:02x} {:02x} {:02x}:{:02x}:{:02x}.{:03x} +0000",
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
            a[6],
            a[7]
        );

        let dt = DateTime::parse_from_str(&b, "%Y %m %d %H:%M:%S%.3f %z");
        let a = dt.unwrap();
        a.into()
    }

    /**
    The physical address and size of the initial ramdisk in memory (mapped in the positive address range).
    */
    pub fn initrd_location() -> (u64, usize) {
        (
            Self::get_bootboot().initrd_ptr,
            Self::get_bootboot().initrd_size as _,
        )
    }

    /**
    Given a physical offset of the memory, return the initrd data.
    In the bootboot context, the offset is 0 because RAM is identity mapped.
    */
    pub fn initrd_data(offset: usize) -> &'static [u8] {
        unsafe {
            core::slice::from_raw_parts(
                (Self::get_bootboot().initrd_ptr as usize + offset) as *const u8,
                Self::get_bootboot().initrd_size as _,
            )
        }
    }

    /**
    Information about Framebuffer.
    */
    pub fn fb_info() -> FrameBufferInfo {
        FrameBufferInfo {
            physical_address: Self::get_bootboot().fb_ptr,
            size: Self::get_bootboot().fb_size as _,
            height: Self::get_bootboot().fb_height as _,
            scanline: Self::get_bootboot().fb_scanline as _,
            width: Self::get_bootboot().fb_width as _,
            format: Self::frame_buffer_format(),
        }
    }

    /**
    Get the memory map information.
    Platform independent and sorted by address memory map.
    */
    pub fn get_mmap_entries() -> &'static [mmap::MemoryMapInfo] {
        Self::get_bootboot().get_mmap_entries()
    }
}

#[cfg(target_arch = "x86_64")]
#[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
impl BootBoot {
    /**
    Physical address of the ACPI information.
    */
    pub fn acpi_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.x86_64.acpi_ptr as usize }
    }

    pub fn smbi_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.x86_64.smbi_ptr as usize }
    }

    pub fn efi_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.x86_64.efi_ptr as usize }
    }

    pub fn mp_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.x86_64.mp_ptr as usize }
    }
}

#[cfg(target_arch = "aarch64")]
#[cfg_attr(docsrs, doc(cfg(target_arch = "aarch64")))]
impl BootBoot {
    /**
    Physical address of the ACPI information.
    */
    pub fn acpi_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.aarch64.acpi_ptr as usize }
    }

    /**
    Physical address of the BCM2837 MMIO.
    */
    pub fn mmio_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.aarch64.mmio_ptr as usize }
    }

    pub fn efi_ptr() -> usize {
        unsafe { Self::get_bootboot().arch.aarch64.efi_ptr as usize }
    }
}

impl BootBoot {
    fn get_bootboot() -> &'static bootboot_bindings::BOOTBOOT {
        unsafe { &(*(bootboot_bindings::BOOTBOOT_INFO as *const bootboot_bindings::BOOTBOOT)) }
    }

    /**
    The frame buffer format.
    */
    fn frame_buffer_format() -> FrameBufferFormat {
        match Self::get_bootboot().fb_type {
            0 => FrameBufferFormat::ARGB,
            1 => FrameBufferFormat::RBGA,
            2 => FrameBufferFormat::ABGR,
            3 => FrameBufferFormat::BGRA,
            _ => FrameBufferFormat::Unknown,
        }
    }
}

/**
Frame buffer format.

The most common is [`FrameBufferFormat::ARGB`]  where the
least significant byte is blue, and the most significant one is skipped (as alpha channel is not used on
lfb) in little-endian order.
*/
#[repr(u8)]
pub enum FrameBufferFormat {
    ARGB = 0,
    RBGA,
    ABGR,
    BGRA,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    /**
     PROTOCOL_MINIMAL is used for embedded systems where environment is not
    implemented, all values and addresses are hardcoded and the frame buffer may not exists at all.
    */
    Minimal,
    /**
    A loader that implements protocol level 1, maps the kernel and the other parts at static locations in
    accordance with the linker
    */
    Static,
    /**
    A level 2 dynamic loader on the other hand generates memory mapping according what’s specified in
    the kernel’s symbol table. It only differs from level 1 that the addresses are flexible.
    */
    Dynamic,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

/**
The loader for the system.
*/
#[derive(Debug, Clone, Copy)]
pub enum Loader {
    Bios,
    Uefi,
    Rpi,
    CoreBoot,
    Unknown,
}

/**
Information about the frame buffer.

Screen coordinates (X, Y) should be converted to offset as:
`offset = (info.height – 1 – Y) * info.scanline + 4 * X`
*/
pub struct FrameBufferInfo {
    /**
    Frame buffer physical address.
    */
    pub physical_address: u64,
    /**
    Frame buffer size.
    */
    pub size: usize,
    /**
    Frame buffer resolution width.
    */
    pub width: usize,
    /**
    Frame buffer resolution height.
    */
    pub height: usize,
    /**
    Frame buffer's bytes per line as stored in memory.
    */
    pub scanline: usize,
    /**
    Frame buffer format.
    */
    pub format: FrameBufferFormat,
}
