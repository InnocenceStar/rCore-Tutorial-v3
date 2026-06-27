//! Constants used in rCore for qemu

pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];
