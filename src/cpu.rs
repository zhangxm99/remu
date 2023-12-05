use crate::bus::Bus;
use crate::exceptions::*;
use crate::param::*;
use crate::csr::*;
use crate::interrupt::*;

pub const MACHINE:u32 = 3;
pub const SUPERVISOR:u32 = 1;
pub const USER:u32 = 0;

pub struct Cpu{
    pc: u32,
    regs: [u32;32],
    bus: Bus,
    csr: Csr,
    mode: u32
}

impl Cpu{
    pub fn new() -> Self{
        let mut regs = [0;32];
        regs[2] = DRAM_END;
        Self{
            pc:DRAM_BASE,
            regs,
            bus:Bus::new(),
            csr:Csr::new(),
            mode:MACHINE
        }
    }
    pub fn reset(&mut self){
        self.regs.iter_mut().for_each(|x| *x = 0);
        self.pc = DRAM_BASE;
        self.regs[2] = DRAM_END;
    }
    pub fn run(&mut self) -> Result<(),Exception>{
        loop{
            let instr = match self.fetch(){
                Ok(instr) => instr,
                Err(e) => {
                    self.handle_exception(e);
                    if e.is_fatal(){
                        println!("{}",e);
                        break Err(e);
                    }
                    continue;
                }
            };
            match self.execute(instr){
                Ok(new_pc) => {
                    self.pc = new_pc;
                },
                Err(e) => {
                    self.handle_exception(e);
                    if e.is_fatal(){
                        println!("{}",e);
                        break Err(e);
                    }
                    continue;
                }
            }
            match self.check_pending_interrupt(){
                Some(i) => self.handle_interrupt(i),
                None => ()
            }
            self.dump_registers();
        }
    }
    pub fn load_binary(&mut self,filename:&str) -> Result<(),Exception>{
        self.bus.load_binary(filename)
    }
    fn dump_registers(&self){
        println!("pc: {:x}",self.pc);
        self.regs
        .iter()
        .zip(0..)
        .for_each(|(x,i)| if (i+1) % 4 == 0 {println!("x{:<2}: {:<9x}",i,x)} else {print!("x{:<2}: {:<9x}",i,x)});
        println!("");
    }
    fn fetch(&self) -> Result<u32,Exception>{
        self.bus.load(self.pc,32)
    }
    fn load(&self,addr:u32,size:u32) -> Result<u32,Exception>{
        self.bus.load(addr,size)
    }
    fn store(&mut self,addr:u32,size:u32,value:u32) -> Result<(),Exception>{
        self.bus.store(addr,size,value)
    }
    fn update_pc(&mut self) -> Result<u32,Exception>{
        Ok(self.pc+4)
    }
    fn handle_exception(&mut self,e:Exception) {
        let mode = self.mode;
        //if exception happened in User or Supervisor level and allowed to be delegate
        let (STATUS,TVEC,CAUSE,EPC,TVAL,MASK_PIE,pie_i,MASK_IE,ie_i,MASK_PP,pp_i) = 
        if mode <= SUPERVISOR && self.csr.is_medelegate(e.code()){
            self.mode = SUPERVISOR;
            (SSTATUS,STVEC,SCAUSE,SEPC,STVAL,MASK_SPIE,5,MASK_SIE,1,MASK_SPP,8)
        } else{
            self.mode = MACHINE;
            (MSTATUS,MTVEC,MCAUSE,MEPC,MTVAL,MASK_MPIE,7,MASK_MIE,3,MASK_MPP,11)
        };
        self.csr.store(CAUSE,e.code());
        self.csr.store(EPC,self.pc);
        if let Ok(tvec) = self.csr.load(TVEC){
            self.pc = tvec;
        }
        self.csr.store(TVAL,e.value());
        if let Ok(mut status) = self.csr.load(STATUS){
            let ie = (status & MASK_IE) >> ie_i;
            status = (status & !MASK_IE) | (ie_i << pie_i);
            status &= !MASK_IE;
            status = (status & !MASK_PP) | (mode << pp_i);
            self.csr.store(STATUS,status);
        };

    }
    fn handle_interrupt(&mut self,i:Interrupt){
        let pc = self.pc;
        let mode = self.mode;
        let (status,tvec,cause_csr,epc,tval,MASK_PIE,pie_i,MASK_PP,pp_i,MASK_IE,ie_i) = 
        if mode <= SUPERVISOR && self.csr.is_midelegate(i.code()){
            self.mode = SUPERVISOR;
            (SSTATUS,STVEC,SCAUSE,SEPC,STVAL,MASK_SPIE,5,MASK_SPP,8,MASK_SIE,1)
        } else{
            self.mode = MACHINE;
            (MSTATUS,MTVEC,MCAUSE,MEPC,MTVAL,MASK_MPIE,7,MASK_MPP,11,MASK_MIE,3)
        };
        self.csr.store(epc,self.pc);
        //according the mode
        let vec = self.csr.load(tvec).unwrap();
        let t_mode = vec & 0b11;
        let t_addr = vec & !0b11;
        self.pc = match mode{
            0 => t_addr,
            1 => t_addr + i.code() << 2,
            _ => unreachable!()
        };
        self.csr.store(cause_csr,i.code());
        self.csr.store(tval,0);
        let mut new_status = self.csr.load(status).unwrap();
        new_status = (new_status & !MASK_PIE) | (MASK_PIE | (((new_status & MASK_IE)>>ie_i) << pie_i));
        new_status = (new_status & !MASK_PP) | (mode << pp_i);
        new_status &= !MASK_IE;
        self.csr.store(status,new_status);

    }
    fn check_pending_interrupt(&mut self) -> Option<Interrupt>{
        let (mstatus,sstatus) = (self.csr.load(MSTATUS).unwrap(),self.csr.load(SSTATUS).unwrap());
        if self.mode == MACHINE && (mstatus & MASK_MIE) == 0{
            return None;
        }
        if self.mode == SUPERVISOR && (sstatus & MASK_SIE) == 0{
            return None;
        }
        
        // if self.bus.uart.is_interrupting(){

        // }
        let (mie,mip) = (self.csr.load(MIE).unwrap(),self.csr.load(MIP).unwrap());
        let pending = mie & mip;

        if (pending & MASK_MEIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_MEIP);
            return Some(Interrupt::MachineExternalInterrupt);
        }
        if (pending & MASK_MSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_MSIP);
            return Some(Interrupt::MachineSoftwareInterrupt);
        }
        if (pending & MASK_MTIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_MTIP);
            return Some(Interrupt::MachineTimerInterrupt);
        }
        if (pending & MASK_SEIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_SEIP);
            return Some(Interrupt::SupervisorExternalInterrupt);
        }
        if (pending & MASK_SSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_SSIP);
            return Some(Interrupt::SupervisorSoftwareInterrupt);
        }
        if (pending & MASK_STIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP).unwrap() & !MASK_STIP);
            return Some(Interrupt::SupervisorTimerInterrupt);
        }
        None
    }

    fn execute(&mut self,inst:u32) -> Result<u32,Exception>{
        let opcode = inst & 0x7f;
        let rd = ((inst & 0xf80) >> 7) as usize;
        let funct3 = (inst & 0x00007000) >> 12;
        let rs1 = ((inst & 0x000f8000) >> 15) as usize;
        let rs2 = ((inst & 0x01f00000) >> 20) as usize;
        let funct7 = (inst & 0xfe000000) >> 25;

        self.regs[0] = 0;
        match opcode {
            0x03 => {
                // imm[11:0] = inst[31:20]
                let imm = (inst as i32 >> 20) as u32;
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {
                        // lb
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val as i8 as i32 as u32;
                        return self.update_pc();
                    }
                    0x1 => {
                        // lh
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val as i16 as i32 as u32;
                        return self.update_pc();
                    }
                    0x2 => {
                        // lw
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val as i32 as u32;
                        return self.update_pc();
                    }
                    0x4 => {
                        // lbu
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    0x5 => {
                        // lhu
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
                }
            }
            0x0f => {
                // A fence instruction does nothing because this emulator executes an
                // instruction sequentially on a single thread.
                match funct3 {
                    0x0 => { // fence
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x13 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst & 0xfff00000) as i32 >> 20) as u32;
                // "The shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I."
                let shamt = (imm & 0x3f) as u32;
                match funct3 {
                    0x0 => {
                        // addi
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm);
                        return self.update_pc();
                    }
                    0x1 => {
                        // slli
                        self.regs[rd] = self.regs[rs1] << shamt;
                        return self.update_pc();
                    }
                    0x2 => {
                        // slti
                        self.regs[rd] = if (self.regs[rs1] as i64) < (imm as i64) { 1 } else { 0 };
                        return self.update_pc();
                    }
                    0x3 => {
                        // sltiu
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                        return self.update_pc();
                    }
                    0x4 => {
                        // xori
                        self.regs[rd] = self.regs[rs1] ^ imm;
                        return self.update_pc();
                    }
                    0x5 => {
                        match funct7 >> 1 {
                            // srli
                            0x00 => {
                                self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                                return self.update_pc();
                            },
                            // srai
                            0x10 => {
                                self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u32;
                                return self.update_pc();
                            }
                            _ => Err(Exception::IllegalInstruction(inst)),
                        }
                    }
                    0x6 => {
                        self.regs[rd] = self.regs[rs1] | imm;
                        return self.update_pc();
                    }, // ori
                    0x7 => {
                        self.regs[rd] = self.regs[rs1] & imm; // andi
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x17 => {
                // auipc
                let imm = (inst & 0xfffff000) as i32 as i64 as u32;
                self.regs[rd] = self.pc.wrapping_add(imm);
                return self.update_pc();
            }
            0x1b => {
                let imm = ((inst as i32 as i64) >> 20) as u32;
                // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                let shamt = (imm & 0x1f) as u32;
                match funct3 {
                    0x0 => {
                        // addiw
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm) as i32 as i64 as u32;
                        return self.update_pc();
                    }
                    0x1 => {
                        // slliw
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt) as i32 as i64 as u32;
                        return self.update_pc();
                    }
                    0x5 => {
                        match funct7 {
                            0x00 => {
                                // srliw
                                self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32
                                    as i64 as u32;
                                return self.update_pc();
                            }
                            0x20 => {
                                // sraiw
                                self.regs[rd] =
                                    (self.regs[rs1] as i32).wrapping_shr(shamt) as i32 as u32;
                                return self.update_pc();
                            }
                            _ => Err(Exception::IllegalInstruction(inst)),
                        }
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
                }
            }
            0x23 => {
                // imm[11:5|4:0] = inst[31:25|11:7]
                let imm = (((inst & 0xfe000000) as i32 as i64 >> 20) as u32) | ((inst >> 7) & 0x1f);
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {self.store(addr, 8, self.regs[rs2])?;  self.update_pc()}, // sb
                    0x1 => {self.store(addr, 16, self.regs[rs2])?; self.update_pc()}, // sh
                    0x2 => {self.store(addr, 32, self.regs[rs2])?; self.update_pc()}, // sw
                    _ => unreachable!(),
                }
            }
            0x2f => {
                let funtc5 = (funct7 & 0b1111100) >> 2;
                let _aq = (funct7 & 0b0000010) >> 1; // acquire access
                let _rl = funct7 & 0b0000001; // release access
                match funtc5{
                    0x00 => {
                        //AMOADD.W 
                        let t = self.load(self.regs[rs1],32)?;
                        self.store(self.regs[rs1],32,t.wrapping_add(self.regs[rs2]));
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x01 => {
                        //AMOSWAP.w
                        let t = self.load(self.regs[rs1],32)?;
                        self.store(self.regs[rs1],32,self.regs[rs2]);
                        self.regs[rs2] = t;
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x04 => {
                        //AMOXOR.W
                        let t = self.load(self.regs[rs1],32)?;
                        self.store(self.regs[rs1],32,t ^ self.regs[rs2]);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x08 => {
                        //AMOOR.W
                        let t = self.load(self.regs[rs1],32)?;
                        self.store(self.regs[rs1],32,t | self.regs[rs2]);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x0c => {
                        //AMOAND.W
                        let t = self.load(self.regs[rs1],32)?;
                        self.store(self.regs[rs1],32,t & self.regs[rs2]);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x10 => {
                        //AMOMIN.w
                        let t = self.load(self.regs[rs1],32)?;
                        if (self.regs[rs2] as i32) < (t as i32) {
                            self.store(self.regs[rs1],32,self.regs[rs2]);
                        }
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x14 => {
                        //AMOMAX.w
                        let t = self.load(self.regs[rs1],32)?;
                        if (self.regs[rs2] as i32) > (t as i32) {
                            self.store(self.regs[rs1],32,self.regs[rs2]);
                        }
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x18 => {
                        //AMOMINU.w
                        let t = self.load(self.regs[rs1],32)?;
                        if self.regs[rs2] < t {
                            self.store(self.regs[rs1],32,self.regs[rs2]);
                        }
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x1c => {
                        //AMOMAXU.w
                        let t = self.load(self.regs[rs1],32)?;
                        if self.regs[rs2] > t {
                            self.store(self.regs[rs1],32,self.regs[rs2]);
                        }
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst))
                }
            }
            0x33 => {
                // "SLL, SRL, and SRA perform logical left, logical right, and arithmetic right
                // shifts on the value in register rs1 by the shift amount held in register rs2.
                // In RV64I, only the low 6 bits of rs2 are considered for the shift amount."
                let shamt = ((self.regs[rs2] & 0x3f) as u32) as u32;
                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // add
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
                        return self.update_pc();
                    }
                    (0x0, 0x01) => {
                        // mul
                        self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
                        return self.update_pc();
                    }
                    (0x1, 0x01) => {
                        //mulh
                        let res:i64 = self.regs[rs1] as i32 as i64 * self.regs[rs2] as i32 as i64;
                        self.regs[rd] = (res >> 32) as u32;
                        return self.update_pc();
                    }
                    (0x2, 0x01) => {
                        //mulhsu
                        let res:i64 = self.regs[rs1] as i32 as i64 * self.regs[rs2] as u64 as i64;
                        self.regs[rd] = (res >> 32) as u32;
                        return self.update_pc();
                    }
                    (0x3, 0x01) => {
                        //mulhu
                        let res:u64 = self.regs[rs1] as u64 * self.regs[rs2] as u64;
                        self.regs[rd] = (res >> 32) as u32;
                        return self.update_pc();
                    }
                    (0x4, 0x01) => {
                        //div
                        if self.regs[rs2] == 0{
                            self.regs[rd] = 0;
                        } else{
                            self.regs[rd] = (self.regs[rs1] as i32 / self.regs[rs2] as i32) as u32;
                        }
                        return self.update_pc();
                    }
                    (0x5, 0x01) => {
                        //divu
                        if self.regs[rs2] == 0{
                            self.regs[rd] = 0;
                        } else{
                            self.regs[rd] = self.regs[rs1] / self.regs[rs2];
                        }
                        return self.update_pc();
                    }
                    (0x6, 0x01) => {
                        //rem
                        if self.regs[rs2] == 0{
                            self.regs[rd] = 0;
                        } else{
                            self.regs[rd] = (self.regs[rs1] as i32 % self.regs[rs2] as i32) as u32;
                        }
                        return self.update_pc();
                    }
                    (0x7, 0x01) => {
                        //remu
                        if self.regs[rs2] == 0{
                            self.regs[rd] = 0;
                        } else{
                            self.regs[rd] = self.regs[rs1] / self.regs[rs2];
                        }
                        return self.update_pc();
                    }
                    (0x0, 0x20) => {
                        // sub
                        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                        return self.update_pc();
                    }
                    (0x1, 0x00) => {
                        // sll
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt);
                        return self.update_pc();
                    }
                    (0x2, 0x00) => {
                        // slt
                        self.regs[rd] = if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) { 1 } else { 0 };
                        return self.update_pc();
                    }
                    (0x3, 0x00) => {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
                        return self.update_pc();
                    }
                    (0x4, 0x00) => {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                        return self.update_pc();
                    }
                    (0x5, 0x00) => {
                        // srl
                        self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                        return self.update_pc();
                    }
                    (0x5, 0x20) => {
                        // sra
                        self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u32;
                        return self.update_pc();
                    }
                    (0x6, 0x00) => {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                        return self.update_pc();
                    }
                    (0x7, 0x00) => {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x37 => {
                // lui
                self.regs[rd] = (inst & 0xfffff000) as i32 as u32;
                return self.update_pc();
            }
            0x63 => {
                // imm[12|10:5|4:1|11] = inst[31|30:25|11:8|7]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 19) as u32)
                    | ((inst & 0x80) << 4) // imm[11]
                    | ((inst >> 20) & 0x7e0) // imm[10:5]
                    | ((inst >> 7) & 0x1e); // imm[4:1]

                match funct3 {
                    0x0 => {
                        // beq
                        if self.regs[rs1] == self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x1 => {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x4 => {
                        // blt
                        if (self.regs[rs1] as i32) < (self.regs[rs2] as i32) {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x5 => {
                        // bge
                        if (self.regs[rs1] as i32) >= (self.regs[rs2] as i32) {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x6 => {
                        // bltu
                        if self.regs[rs1] < self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x7 => {
                        // bgeu
                        if self.regs[rs1] >= self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
                }
            }
            0x67 => {
                // jalr
                let t = self.pc + 4;

                let imm = (((inst & 0xfff00000) as i32) >> 20) as u32;
                let new_pc = (self.regs[rs1].wrapping_add(imm)) & !1;

                self.regs[rd] = t;
                return Ok(new_pc);
            }
            0x6f => {
                // jal
                self.regs[rd] = self.pc + 4;

                // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u32) // imm[20]
                    | (inst & 0xff000) // imm[19:12]
                    | ((inst >> 9) & 0x800) // imm[11]
                    | ((inst >> 20) & 0x7fe); // imm[10:1]

                return Ok(self.pc.wrapping_add(imm));
            }
            0x73 => {
                let csr_addr = ((inst & 0xfff00000) >> 20) as usize;
                match funct3{
                    0x0 => {
                        match (rs2,funct7){
                            (0x0,0x0) => {
                                //ecall
                                match self.mode {
                                    USER => Err(Exception::EnvironmentCallFromUMode(self.pc)),
                                    SUPERVISOR => Err(Exception::EnvironmentCallFromSMode(self.pc)),
                                    MACHINE => Err(Exception::EnvironmentCallFromMMode(self.pc)),
                                    _ => unreachable!()
                                }
                            }
                            (0x1,0x0) => {
                                //ebreak
                                return Err(Exception::Breakpoint(self.pc));
                            }
                            (0x2,0x8) => {
                                //sret
                                //mode <- MPP, SIE <- MPIE, MPP <- 0, MIE <- 0,pc <- SEPC
                                let mut sstatus = self.csr.load(SSTATUS)?;
                                let spp = (sstatus | MASK_MPP) >> 11;
                                let spie = (sstatus | MASK_SPIE) >> 7;
                                self.mode = spp;
                                sstatus = (sstatus & !MASK_SIE) | (spie << 1);
                                sstatus &= !MASK_SPP;
                                sstatus &= !MASK_SPIE;
                                let new_pc = self.csr.load(SEPC)? & !0b11;
                                Ok(new_pc)
                            }
                            (0x2,0x18) => {
                                //mret
                                //mode <- MPP, MIE <- MPIE, MPP <- 0, MIE <- 0,pc <- MEPC
                                let mut mstatus = self.csr.load(MSTATUS)?;
                                let mpp = (mstatus | MASK_SPP) >> 8;
                                let mpie = (mstatus | MASK_SPIE) >> 5;
                                self.mode = mpp;
                                mstatus = (mstatus & !MASK_SIE) | (mpie << 3);
                                mstatus &= !MASK_MPP;
                                mstatus &= !MASK_MPIE;
                                let new_pc = self.csr.load(MEPC)? & !0b11;
                                Ok(new_pc)
                            }
                            (_, 0x9) => {
                                // sfence.vma
                                // Do nothing.
                                return self.update_pc();
                            }
                            _ => Err(Exception::IllegalInstruction(inst))
                        }
                    }
                    0x1 => {
                        //csrrw
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,self.regs[rs1]);
                        return self.update_pc();
                    }
                    0x2 => {
                        //csrrs
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,self.regs[rs1] | self.regs[rd]);
                        return self.update_pc();
                    }
                    0x3 => {
                        //csrrc
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,!self.regs[rs1] & self.regs[rd]);
                        return self.update_pc();
                    }
                    0x5 => {
                        //csrrwi
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,rs1 as u32);
                        return self.update_pc();
                    }
                    0x6 => {
                        //csrrsi
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,(rs1 as u32) | self.regs[rd]);
                        return self.update_pc();
                    }
                    0x7 => {
                        //csrrci
                        self.regs[rd] = self.csr.load(csr_addr)?;
                        self.csr.store(csr_addr,!(rs1 as u32) & self.regs[rd]);
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst))
                }
            }

            _ => Err(Exception::IllegalInstruction(inst)),
        }
    }


}