use core::mem::size_of;

#[allow(unused)]
#[repr(C)]
pub struct MultibootInfo {
    /* Multiboot info version number */
    flags: u32,

    /* Available memory from BIOS */
    mem_lower: u32,
    mem_upper: u32,

    /* "root" partition */
    boot_device: u32,

    /* Kernel command line */
    cmdline: u32,

    /* Boot-Module list */
    mods_count: u32,
    mods_addr: u32,

    /*
          union
          {
            multiboot_aout_symbol_table_t aout_sym;
            multiboot_elf_section_header_table_t elf_sec;
          } u;
        struct multiboot_elf_section_header_table
    {
      multiboot_uint32_t num;
      multiboot_uint32_t size;
      multiboot_uint32_t addr;
      multiboot_uint32_t shndx;

    };
        */
    padding1: [u8; 16],
    /* Memory Mapping buffer */
    mmap_length: u32,
    mmap_addr: u32,

    /* Drive Info buffer */
    drives_length: u32,
    drives_addr: u32,

    /* ROM configuration table */
    config_table: u32,

    /* Boot Loader Name */
    boot_loader_name: *const u8,

    /* APM table */
    apm_table: u32,

    /* Video */
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u16,
    vbe_interface_seg: u16,
    vbe_interface_off: u16,
    vbe_interface_len: u16,

    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    /*  #define MULTIBOOT_FRAMEBUFFER_TYPE_INDEXED 0
    #define MULTIBOOT_FRAMEBUFFER_TYPE_RGB     1
    #define MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT     2 */
    framebuffer_type: u8,

    padding2: [u8; 6],
}

impl MultibootInfo {
    pub fn check_flags_for_memmap(&self) -> bool {
        (self.flags >> 6 & 0x1) == 0
    }

    pub unsafe fn get_mmap_addrs(&self) -> &[MultibootMMapEntry] {
        let num_mmap_addr = self.mmap_length as usize / size_of::<MultibootMMapEntry>();

        core::slice::from_raw_parts(self.mmap_addr as *const MultibootMMapEntry, num_mmap_addr)
    }

    pub fn loop_through_memory_map(&self, f: fn(&MultibootMMapEntry)) {
        unsafe {
            for mmap in self.get_mmap_addrs() {
                f(&mmap);
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
#[repr(C, packed)]
pub struct MultibootMMapEntry {
    size: u32,
    addr: u64,
    len: u64,
    r#type: u32,
}

impl MultibootMMapEntry {
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn r#type(&self) -> u32 {
        self.r#type
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub enum MultibootMemoryMappedType {
    Available,
    Reserved,
    AcpiReclaimable,
    Nvs,
    BadRam,
    InvalidData,
}

impl MultibootMemoryMappedType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Available,
            2 => Self::Reserved,
            3 => Self::AcpiReclaimable,
            4 => Self::Nvs,
            5 => Self::BadRam,
            _ => Self::InvalidData,
        }
    }
}

pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;
