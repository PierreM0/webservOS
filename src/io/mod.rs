use core::arch::asm;

pub mod serial;
pub mod vga;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe{
            use core::fmt::Write as FmtWrite;
            let mut writer = $crate::io::vga::VGA_WRITER.borrow_mut();
            write!(writer, $($arg)*).expect("Failed to print to vga");

            let mut writer = $crate::io::serial::SerialWriter::new($crate::io::serial::PORT);
            write!(writer, $($arg)*).expect("Failed to print to serial");
        }
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        {
            print!($($arg)*);
            print!("\n");
        }
    }
}

pub mod pci;

unsafe fn outb(address: u16, value: u8) {
    asm!(r#"
        .att_syntax
         out %al, %dx
         "#,
        in("al") value,
        in("dx") address
    );
}

unsafe fn inb(address: u16) -> u8 {
    let mut ret;
    asm!(r#"
        .att_syntax
        in %dx, %al
        "#,
        in("dx") address,
        out("al") ret);
    ret
}

unsafe fn outl(address: u16, value: u32) {
    asm!(r#"
        .att_syntax
         out %eax, %dx
         "#,
        in("eax") value,
        in("dx") address
    );
}

unsafe fn inl(address: u16) -> u32 {
    let mut ret;
    asm!(r#"
        .att_syntax
        in %dx, %eax
        "#,
        in("dx") address,
        out("eax") ret);
    ret
}
