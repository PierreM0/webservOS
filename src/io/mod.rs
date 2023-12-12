pub mod serial;
pub mod vga;

macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe{
            use core::fmt::Write as FmtWrite;
            let mut writer = $crate::io::vga::VGA_WRITER.borrow_mut();
            write!(writer, $($arg)*).expect("Failed to print to vga");

            let mut writer = $crate::io::serial::SerialWriter::new(serial::PORT);
            write!(writer, $($arg)*).expect("Failed to print to serial");
        }
    }
}

macro_rules! println {
    ($($arg:tt)*) => {
        {
            print!($($arg)*);
            print!("\n");
        }
    }
}
