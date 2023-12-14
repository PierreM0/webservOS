use core::{cell::RefCell, fmt::Write};

pub static VGA_WRITER: StaticVGATerminalWriter = StaticVGATerminalWriter::new();

pub struct StaticVGATerminalWriter {
    inner: RefCell<VGATerminalWriter>,
}

impl core::ops::Deref for StaticVGATerminalWriter {
    type Target = RefCell<VGATerminalWriter>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// I only have one thread
unsafe impl Sync for StaticVGATerminalWriter {}

impl StaticVGATerminalWriter {
    const fn new() -> StaticVGATerminalWriter {
        StaticVGATerminalWriter {
            inner: RefCell::new(VGATerminalWriter::new()),
        }
    }
}

impl Write for StaticVGATerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write!(self.inner.borrow_mut(), "{s}")
    }
}

#[allow(dead_code)]
#[repr(u8)]
enum VgaColor {
    VgaColorBlack = 0,
    VgaColorBlue = 1,
    VgaColorGreen = 2,
    VgaColorCyan = 3,
    VgaColorRed = 4,
    VgaColorMagenta = 5,
    VgaColorBrown = 6,
    VgaColorLightGrey = 7,
    VgaColorDarkGrey = 8,
    VgaColorLightBlue = 9,
    VgaColorLightGreen = 10,
    VgaColorLightCyan = 11,
    VgaColorLightRed = 12,
    VgaColorLightMagenta = 13,
    VgaColorLightBrown = 14,
    VgaColorWhite = 15,
}

const fn vga_entry_color(fg: u8, bg: u8) -> u8 {
    fg | bg << 4
}

const fn vga_entry(uc: u8, color: u8) -> u16 {
    uc as u16 | (color as u16) << 8
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

pub struct VGATerminalWriter {
    row: usize,
    column: usize,
    color: u8,
    buffer: *mut u16,
}

unsafe impl Sync for VGATerminalWriter {}

impl Write for VGATerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

pub fn init() {
    let vtw = VGA_WRITER.borrow_mut();
    for y in 0..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            let index = y * VGA_WIDTH + x;
            unsafe {
                *vtw.buffer.add(index) = vga_entry(b' ', vtw.color);
            }
        }
    }
}

impl VGATerminalWriter {
    pub const fn new() -> Self {
        let row = 0;
        let column = 0;
        let color = vga_entry_color(
            VgaColor::VgaColorLightGrey as u8,
            VgaColor::VgaColorBlack as u8,
        );
        let buffer = 0xB8000 as *mut u16;
        Self {
            row,
            column,
            color,
            buffer,
        }
    }

    #[allow(dead_code)]
    fn setcolor(&mut self, color: u8) {
        self.color = color;
    }

    fn putentryat(&mut self, c: u8, color: u8, x: usize, y: usize) {
        let index = y * VGA_WIDTH + x;
        unsafe {
            *(self.buffer).add(index) = vga_entry(c, color);
        }
    }

    pub fn putchar(&mut self, c: u8) {
        if self.row >= VGA_HEIGHT {
            self.row = 0; // FIXME
        }
        if c == b'\n' {
            self.row += 1;
            self.column = 0;
        } else {
            self.putentryat(c, self.color, self.column, self.row);
            self.column += 1;
            if self.column == VGA_WIDTH {
                self.column = 0;
                self.row += 1;
                if self.row == VGA_HEIGHT {
                    self.row = 0;
                }
            }
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        for c in data {
            self.putchar(*c);
        }
    }
}
