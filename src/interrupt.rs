pub const MASK_INTERRUPT_BIT: u32 = 1 << 31;

#[derive(Copy,Clone,Debug)]
pub enum Interrupt{
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt
}

use Interrupt::*;
impl Interrupt{
    pub fn code(self) -> u32{
        match self{
            SupervisorSoftwareInterrupt => MASK_INTERRUPT_BIT | 1,
            MachineSoftwareInterrupt => MASK_INTERRUPT_BIT | 3,
            SupervisorTimerInterrupt => MASK_INTERRUPT_BIT | 5,
            MachineTimerInterrupt => MASK_INTERRUPT_BIT | 7,
            SupervisorExternalInterrupt => MASK_INTERRUPT_BIT | 9,
            MachineExternalInterrupt => MASK_INTERRUPT_BIT | 11,
        }
    }
}