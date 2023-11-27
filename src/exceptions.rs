use std::fmt;

pub enum Exception {
    // Riscv Standard Exception
    InstructionAddrMisaligned(u32),
    InstructionAccessFault(u32),
    IllegalInstruction(u32),
    Breakpoint(u32),
    LoadAccessMisaligned(u32),
    LoadAccessFault(u32),
    StoreAMOAddrMisaligned(u32),
    StoreAMOAccessFault(u32),
    EnvironmentCallFromUMode(u32),
    EnvironmentCallFromSMode(u32),
    EnvironmentCallFromMMode(u32),
    InstructionPageFault(u32),
    LoadPageFault(u32),
    StoreAMOPageFault(u32),
}

use Exception::*;
impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionAddrMisaligned(addr) => write!(f, "Instruction address misaligned {:#x}", addr),
            InstructionAccessFault(addr) => write!(f, "Instruction access fault {:#x}", addr),
            IllegalInstruction(inst) => write!(f, "Illegal instruction {:#x}", inst),
            Breakpoint(pc) => write!(f, "Breakpoint {:#x}", pc),
            LoadAccessMisaligned(addr) => write!(f, "Load access {:#x}", addr),
            LoadAccessFault(addr) => write!(f, "Load access fault {:#x}", addr),
            StoreAMOAddrMisaligned(addr) => write!(f, "Store or AMO address misaliged {:#x}", addr),
            StoreAMOAccessFault(addr) => write!(f, "Store or AMO access fault {:#x}", addr),
            EnvironmentCallFromUMode(pc) => write!(f, "Environment call from U-mode {:#x}", pc),
            EnvironmentCallFromSMode(pc) => write!(f, "Environment call from S-mode {:#x}", pc),
            EnvironmentCallFromMMode(pc) => write!(f, "Environment call from M-mode {:#x}", pc),
            InstructionPageFault(addr) => write!(f, "Instruction page fault {:#x}", addr),
            LoadPageFault(addr) => write!(f, "Load page fault {:#x}", addr),
            StoreAMOPageFault(addr) => write!(f, "Store or AMO page fault {:#x}", addr),
        }
    }
}
