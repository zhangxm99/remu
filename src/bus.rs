use crate::dram::Dram;
use crate::exceptions::Exception;
use crate::param::*;
pub struct Bus{
    pub dram: Dram
}

impl Bus{
    pub fn new()->Self{
        Self{dram:Dram::new()}
    }
    pub fn load_binary(&mut self,filename:&str) -> Result<(),Exception>{
        self.dram.load_instruction(filename)
    }
    pub fn load(&self,addr:u32,size:u32) -> Result<u32,Exception>{
        match addr{
            DRAM_BASE..=DRAM_END => self.dram.load(addr,size),
            _ => Err(Exception::LoadAccessFault(addr))
        }
    }
    pub fn store(&mut self,addr:u32,size:u32,value:u32) -> Result<(),Exception>{
        match addr{
            DRAM_BASE..=DRAM_END => self.dram.store(addr,size,value),
            _ => Err(Exception::LoadAccessFault(addr))
        }
    }

}

