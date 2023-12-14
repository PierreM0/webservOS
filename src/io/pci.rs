use super::{inl, outl};
use alloc::vec::Vec;

// https://wiki.osdev.org/PCI
const NOT_A_VENDOR: u16 = 0xFFFF;
const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

trait GetByte {
    fn nth_byte(&self, n: usize) -> u8;
}

impl GetByte for u32 {
    fn nth_byte(&self, n: usize) -> u8 {
        (self >> 8 * n & 0xFF) as u8
    }
}

impl GetByte for u16 {
    fn nth_byte(&self, n: usize) -> u8 {
        (self >> 8 * n & 0xFF) as u8
    }
}

#[derive(Debug)]
struct PciAddr {
    bus: u8,
    slot: u8,
    function: u8,
}

impl PciAddr {
    fn get_vendor_id(&self) -> u16 {
        unsafe { pci_config_read_word(self.bus, self.slot, self.function, 0) }
    }

    fn get_device_id(&self) -> u16 {
        unsafe { pci_config_read_word(self.bus, self.slot, self.function, 2) }
    }

    fn get_secondary_bus(&self) -> u8 {
        unsafe { pci_config_read_word(self.bus, self.slot, self.function, 0x18 + 0x1).nth_byte(0) }
    }

    fn get_sub_class(&self) -> u8 {
        unsafe { pci_config_read_word(self.bus, self.slot, self.function, 0x8 + 0x2).nth_byte(0) }
    }

    fn get_base_class(&self) -> u8 {
        unsafe { pci_config_read_word(self.bus, self.slot, self.function, 0x8 + 0x3).nth_byte(1) }
    }

    fn get_header_type(&self) -> u8 {
        unsafe {
            (pci_config_read_word(self.bus, self.slot, self.function, 0xC + 0x2) & 0x0F) as u8
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
struct PciDeviceHeader {
    device_id: u16,
    vendor_id: u16,
    class: u8,
    subclass: u8,
    addr: PciAddr,
}

unsafe fn pci_config_read_word(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    let lbus = bus as u32;
    let lslot = device as u32;
    let lfunc = func as u32;

    // Create configuration address as per Figure 1
    let address = ((lbus << 16)
        | (lslot << 11)
        | (lfunc << 8)
        | (offset & 0xFC) as u32
        | (0x80000000 as u32)) as u32;

    // Write out the address
    outl(CONFIG_ADDRESS, address);
    // Read in the data
    // (offset & 2) * 8) = 0 will choose the first word of the 32-bit register
    ((inl(CONFIG_DATA) >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

fn check_function(bus: u8, device: u8, function: u8) -> Vec<PciDeviceHeader> {
    let addr = PciAddr {
        bus,
        slot: device,
        function,
    };
    let base_class = addr.get_base_class();
    let sub_class = addr.get_sub_class();
    let vendor_id = addr.get_vendor_id();

    if (base_class == 0x6) && (sub_class == 0x4) {
        let secondary_bus = addr.get_secondary_bus();
        return check_bus(secondary_bus);
    }

    let mut res = Vec::new();
    res.push(PciDeviceHeader {
        device_id: addr.get_device_id(),
        vendor_id,
        class: base_class,
        subclass: sub_class,
        addr,
    });
    res
}

fn check_device(bus: u8, device: u8) -> Vec<PciDeviceHeader> {
    let function = 0;
    let addr = PciAddr {
        bus,
        slot: device,
        function,
    };
    let vendor_id = addr.get_vendor_id();
    if vendor_id == NOT_A_VENDOR {
        return Vec::new();
    }

    let mut res = Vec::new();
    res.append(&mut check_function(bus, device, function));

    let header_type = addr.get_header_type();
    if (header_type & 0x80) != 0 {
        // multi function device
        for function in 1..8 {
            let addr = PciAddr {
                bus,
                slot: device,
                function,
            };
            if addr.get_vendor_id() != NOT_A_VENDOR {
                res.append(&mut check_function(bus, device, function));
            }
        }
    }
    res
}

fn check_bus(bus: u8) -> Vec<PciDeviceHeader> {
    let mut res = Vec::new();
    for device in 0..32 {
        res.append(&mut check_device(bus, device));
    }
    res
}

fn check_all_buses_smart() -> Vec<PciDeviceHeader> {
    let addr = PciAddr {
        bus: 0,
        slot: 0,
        function: 0,
    };
    let header_type = addr.get_header_type();
    if (header_type & 0x80) == 0 {
        // single PCI host controller
        check_bus(0)
    } else {
        // multiple PCI host contoller
        let mut res = Vec::new();
        for function in 0..8 {
            let addr = PciAddr {
                bus: 0,
                slot: 0,
                function,
            };
            if addr.get_vendor_id() != NOT_A_VENDOR {
                let bus = function;
                res.append(&mut check_bus(bus));
            }
        }
        res
    }
}

    let pci_devices_headers = check_all_buses_smart();
    let rtl8139 = pci_devices_headers
        .iter()
        .find(|e| e.device_id == 0x8139 && e.vendor_id == 0x10ec)
        .expect("good qemu config");

    println!("network card: {rtl8139:#1x?}");
