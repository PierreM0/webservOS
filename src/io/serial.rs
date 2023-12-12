use core::{arch::asm, fmt::Write};

pub static PORT: u16 = 0x3f8; // COM1

pub struct SerialWriter {
    port: u16,
}

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

pub struct SerialWriterInitError;

pub fn init() -> Result<(), SerialWriterInitError> {
    unsafe {
        outb(PORT + 1, 0x00); // Disable all interrupts
        outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(PORT + 1, 0x00); //                  (hi byte)
        outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
        outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i: not same byte as sent)
        if inb(PORT + 0) != 0xAE {
            return Err(SerialWriterInitError);
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(PORT + 4, 0x0F);
    }
    Ok(())
}

impl SerialWriter {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    unsafe fn is_transmit_empty(&self) -> u8 {
        return inb(self.port + 5) & 0x20;
    }

    unsafe fn write_serial(&self, a: u8) {
        while self.is_transmit_empty() == 0 {}

        outb(self.port, a);
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.as_bytes() {
            unsafe { self.write_serial(*c) }
        }
        Ok(())
    }
}
