pub const DRAM_BASE: u32 = 0x8000_0000;
pub const DRAM_SIZE: u32 = 512*1024*1024;
pub const DRAM_END : u32 = DRAM_SIZE + DRAM_BASE;

pub const UART_BASE: u32 = 0x1000_0000;
pub const UART_SIZE: u32 = 0x100;
pub const UART_END : u32 = UART_BASE + UART_SIZE;

pub const PTE_SIZE:  u32 = 4;
pub const PAGE_SIZE: u32 = 1024;
