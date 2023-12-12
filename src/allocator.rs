use crate::boot::{MultibootInfo, MultibootMemoryMappedType};
use crate::{io::*, KERNEL_START};
use core::{alloc::GlobalAlloc, mem::size_of};

#[global_allocator]
pub static mut ALLOCATOR: Allocator = Allocator::new();

pub struct Allocator {
    mmap_addr: *mut u8,
    mmap_size: usize,
}

impl Allocator {
    const fn new() -> Self {
        Self {
            mmap_addr: 0 as *mut u8,
            mmap_size: 0,
        }
    }

    pub unsafe fn init(&mut self, multiboot_infos: &'static MultibootInfo) {
        let mmap = multiboot_infos
            .get_mmap_addrs()
            .iter()
            .find(|e| e.addr() == (&crate::KERNEL_START as *const u32) as u64)
            .expect("There is at least one available memory map");

        let kernel_start_addr = (&crate::KERNEL_START as *const u32) as u64;
        let kernel_end_addr = (&crate::KERNEL_END as *const u32) as u64;

        let reserved_memory_length = kernel_end_addr - kernel_start_addr;

        let reallign = (mmap.len() - reserved_memory_length) % 0x4;
        let reserved_memory_length = reserved_memory_length + reallign;

        self.mmap_size = (mmap.len() - reserved_memory_length) as usize;
        self.mmap_addr = (kernel_start_addr + reserved_memory_length) as *mut u8;
        println!("self.mmap_addr {:?}", self.mmap_addr);

        let a = self.mmap_addr;
        let header = Header {
            special: true,
            alloced: false,
            size: self.mmap_size - size_of::<Header>() - size_of::<Header>(),
        };

        a.copy_from_nonoverlapping((&header as *const Header) as *const u8, size_of::<Header>());
        println!("a {:?}", a);
        let a = a.add(self.mmap_size - size_of::<Header>());
        a.copy_from_nonoverlapping((&header as *const Header) as *const u8, size_of::<Header>());
    }
}

//#[repr(packed)]
#[derive(Debug)]
struct Header {
    special: bool,
    alloced: bool,
    size: usize,
}

// [Header => alloced: <bool>, size: <u32>]
// [Allocated item]
// [Header => alloced: <bool>, size: <u32>]

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let size_asked = layout.size();
        let mut a = self.mmap_addr;
        println!("a {:?}", a);
        let mut header = a.cast::<Header>();
        while (*header).alloced || (*header).size < (size_asked + 2 * size_of::<Header>()) {
            if a.addr() > self.mmap_addr as usize && (*header).special {
                panic!("Uh oh... No more memory !")
            }
            a = a
                .add((*header).size)
                .add(size_of::<Header>())
                .add(size_of::<Header>());
            header = a.cast::<Header>();
        }

        let updated_header = Header {
            special: false,
            alloced: true,
            size: size_asked,
        };
        let new_free_header = Header {
            special: false,
            alloced: false,
            size: (*header).size - size_asked - 2 * size_of::<Header>(),
        };

        // at first because the header size value will be rewrote
        let last_free_header = (header as *mut u8)
            .add((*header).size)
            .add(size_of::<Header>()) as *mut Header;

        (*last_free_header).alloced = false;
        (*last_free_header).size = new_free_header.size;
        // dont touch (*last_free_header).special

        (*header).alloced = true;
        (*header).size = updated_header.size;
        //don't touch (*header).special

        let res = a.add(size_of::<Header>());

        let a = res.add(size_asked);
        a.copy_from_nonoverlapping(
            (&updated_header as *const Header) as *const u8,
            size_of::<Header>(),
        );

        let a = a.add(size_of::<Header>());
        a.copy_from_nonoverlapping(
            (&new_free_header as *const Header) as *const u8,
            size_of::<Header>(),
        );

        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        let header = ptr.offset(-1 * size_of::<Header>() as isize);
        let header = header as *mut Header;
        if !(*header).alloced {
            return;
        }

        let header_size = (*header).size;
        (*header).alloced = false;
        let snd_header = ptr.add((*header).size) as *mut Header;
        (*snd_header).alloced = false;

        let next_header_exists = !(*snd_header).special;

        if !(*snd_header).special {
            let next_header = ptr
                .add(header_size) // first header
                .add(size_of::<Header>())
                .cast::<Header>();

            if !(*next_header).alloced {
                // merge
                let next_header = next_header
                    .cast::<u8>()
                    .add((*next_header).size)
                    .add(size_of::<Header>()) as *mut Header;

                // add middle header
                (*header).size += (*next_header).size + 2 * size_of::<Header>();
                (*next_header).size = (*header).size;
            }
        }

        if !(*header).special {
            let last_header =
                (header as *const u8).offset(-1 * size_of::<Header>() as isize) as *mut Header; // second header
            let last_header = (last_header as *const u8)
                .offset(-1 * (*last_header).size as isize)
                .offset(-1 * size_of::<Header>() as isize)
                as *mut Header;

            if !(*last_header).alloced {
                if next_header_exists && {
                    let next_header = ptr.add(header_size).add(size_of::<Header>()) as *mut Header;
                    !(*next_header).alloced
                } {
                    let next_header = ptr.add(header_size).add(size_of::<Header>()) as *mut Header;

                    let next_header = next_header
                        .cast::<u8>()
                        .add((*next_header).size)
                        .add(size_of::<Header>())
                        as *mut Header;

                    // add 4 header
                    // - last from last_header
                    // - 2 from header (but 1 already has been added)
                    // - first from next_header (but has already been added)
                    (*last_header).size += (*next_header).size + 2 * size_of::<Header>();
                    (*next_header).size = (*last_header).size;
                } else {
                    // add middle header
                    (*last_header).size += (*snd_header).size + 2 * size_of::<Header>();
                    (*snd_header).size = (*last_header).size;
                }
            }
        }
    }
}
