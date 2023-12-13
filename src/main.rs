#![feature(custom_test_frameworks)]
#![feature(strict_provenance)]
#![feature(const_mut_refs)]
#![feature(panic_info_message)]
#![feature(const_for)]
#![allow(bad_asm_style)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use alloc::vec;
use tinyrand::Rand;

use crate::io::serial;
use crate::io::vga;
use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

#[macro_use]
mod io;
mod allocator;
mod boot;

extern crate alloc;
// Include boot.s which defines _start as inline assembly in main. This allows us to do more fine
// grained setup than if we used a naked _start function in rust. Theoretically we could use a
// naked function + some inline asm, but this seems much more straight forward.
global_asm!(include_str!("boot.S"));

extern "C" {
    static KERNEL_START: u32;
    static KERNEL_END: u32;
}

#[no_mangle]
pub unsafe extern "C" fn kernel_main(
    multiboot_infos: &'static boot::MultibootInfo,
    multiboot_magic: u32,
) -> ! {
    vga::init();
    let _ = serial::init();
    unsafe { allocator::ALLOCATOR.init(multiboot_infos) }

    let mut rng = tinyrand::StdRand::default();

    for i in 0..100 {
        let mut v = vec![];
        for j in 0..i {
            v.push(j);
        }
        for _ in 0..v.len() {
            let num = rng.next_usize() % v.len();
            v.remove(num);
        }
    }

    let mmaps = multiboot_infos.get_mmap_addrs();
    let mut size = 0;
    for mmap in mmaps {
        println!("{:?}", mmap.r#type());
        size += mmap.len();
    }

    println!("total mmap size: {size}");
    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Pannic'ed on:");
    if let Some(message) = info.message() {
        print!("{}", message);
    }
    if let Some(location) = info.location() {
        print!(
            " at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    println!(".");
    unsafe {
        asm! { "hlt" }
    }
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u8;
    for i in 0..n {
        *s.add(i) = c;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dst.add(i) = *src.add(i);
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let i: isize = i.try_into().expect("i can be transformed into isize");
        let b1 = *s1.offset(i);
        let b2 = *s2.offset(i);

        if b1 != b2 {
            return b1 as i32 - b2 as i32;
        }
    }
    0
}
