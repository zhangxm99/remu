use crate::exceptions::Exception;

pub const NUM_CSRS: usize = 4096;
// Machine-level CSRs.
/// ISA that this cpu supported
pub const MISA: usize = 0x301;
/// Vendor ID
pub const MVendorid: usize = 0xf11;
/// Hardware thread ID.
pub const MHARTID: usize = 0xf14;
/// Machine status register.
pub const MSTATUS: usize = 0x300;
/// Machine exception delefation register.
pub const MEDELEG: usize = 0x302;
/// Machine interrupt delefation register.
pub const MIDELEG: usize = 0x303;
/// Machine interrupt-enable register.
pub const MIE: usize = 0x304;
/// Machine trap-handler base address.
pub const MTVEC: usize = 0x305;
/// Machine counter enable.
pub const MCOUNTEREN: usize = 0x306;
/// Scratch register for machine trap handlers.
pub const MSCRATCH: usize = 0x340;
/// Machine exception program counter.
pub const MEPC: usize = 0x341;
/// Machine trap cause.
pub const MCAUSE: usize = 0x342;
/// Machine bad address or instruction.
pub const MTVAL: usize = 0x343;
/// Machine interrupt pending.
pub const MIP: usize = 0x344;

// Supervisor-level CSRs.
/// Supervisor status register.
pub const SSTATUS: usize = 0x100;
/// Supervisor interrupt-enable register.
pub const SIE: usize = 0x104;
/// Supervisor trap handler base address.
pub const STVEC: usize = 0x105;
/// Scratch register for supervisor trap handlers.
pub const SSCRATCH: usize = 0x140;
/// Supervisor exception program counter.
pub const SEPC: usize = 0x141;
/// Supervisor trap cause.
pub const SCAUSE: usize = 0x142;
/// Supervisor bad address or instruction.
pub const STVAL: usize = 0x143;
/// Supervisor interrupt pending.
pub const SIP: usize = 0x144;
/// Supervisor address translation and protection.
pub const SATP: usize = 0x180;


// mstatus and sstatus field mask
pub const MASK_SIE: u32 = 1 << 1; 
pub const MASK_MIE: u32 = 1 << 3;
pub const MASK_SPIE: u32 = 1 << 5; 
pub const MASK_UBE: u32 = 1 << 6; 
pub const MASK_MPIE: u32 = 1 << 7;
pub const MASK_SPP: u32 = 1 << 8; 
pub const MASK_VS: u32 = 0b11 << 9;
pub const MASK_MPP: u32 = 0b11 << 11;
pub const MASK_FS: u32 = 0b11 << 13; 
pub const MASK_XS: u32 = 0b11 << 15; 
pub const MASK_MPRV: u32 = 1 << 17;
pub const MASK_SUM: u32 = 1 << 18; 
pub const MASK_MXR: u32 = 1 << 19; 
pub const MASK_TVM: u32 = 1 << 20;
pub const MASK_TW: u32 = 1 << 21;
pub const MASK_TSR: u32 = 1 << 22;
pub const MASK_SSTATUS: u32 = MASK_SIE | MASK_SPIE | MASK_UBE | MASK_SPP | MASK_FS 
                            | MASK_XS  | MASK_SUM  | MASK_MXR ;


// MIP / SIP field mask
pub const MASK_SSIP: u32 = 1 << 1;
pub const MASK_MSIP: u32 = 1 << 3;
pub const MASK_STIP: u32 = 1 << 5;
pub const MASK_MTIP: u32 = 1 << 7;
pub const MASK_SEIP: u32 = 1 << 9;
pub const MASK_MEIP: u32 = 1 << 11;

pub struct Csr{
    pub csrs: [u32;NUM_CSRS]
}

impl Csr{
    pub fn new() -> Self{
        Self{csrs:[0;NUM_CSRS]}
    }
    pub fn load(&self,addr:usize) -> Result<u32,Exception>{
        match addr{
            SIE => Ok(self.csrs[MIE] & self.csrs[MIDELEG]),
            SIP => Ok(self.csrs[MIP] & self.csrs[MIDELEG]),
            SSTATUS => Ok(self.csrs[MSTATUS] & MASK_SSTATUS),
            0..=4095 => Ok(self.csrs[addr]),
            _ => Err(Exception::IllegalInstruction(addr as u32))
        }
    }

    pub fn store(&mut self,addr:usize,value:u32) -> Result<(),Exception>{
        match addr{
            SIE => Ok(self.csrs[MIE] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG])),
            SIP => Ok(self.csrs[MIP] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG])),
            SSTATUS => Ok(self.csrs[MSTATUS] = (self.csrs[MSTATUS] & !MASK_SSTATUS) | (value & MASK_SSTATUS)),
            0..=4095 => Ok(self.csrs[addr] = value),
            _ => Err(Exception::IllegalInstruction(addr as u32))
        }
    }

}