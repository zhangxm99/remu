pub const DRAM_BASE: u32 = 0x8000_0000;
pub const DRAM_SIZE: u32 = 512*1024*1024;
pub const DRAM_END : u32 = DRAM_SIZE + DRAM_BASE - 1;