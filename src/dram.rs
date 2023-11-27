use std::fs::read;
use crate::exceptions::Exception;
use crate::param::*;
pub struct Dram{
    pub data:Vec<u8>
}

impl Dram{
    pub fn new() -> Self{
        // code.extend(0..(512*1024*1024-code.len()));
        Self{data:Vec::new()}
    }
    pub fn load_instruction(&mut self,filename:&str) -> Result<(),Exception>{
        self.data = read(filename).expect("Failed to read file");
        self.data.resize(DRAM_SIZE as usize,0);
        Ok(())
    }
    pub fn load(&self,addr:u32,size:u32) -> Result<u32,Exception>{
        if ![8,16,32].contains(&size){
            return Err(Exception::LoadAccessFault(addr));
        }
        let index = (addr - DRAM_BASE) as usize;
        let mut code = self.data[index] as u32;
        let nbytes = size / 8;
        for i in 1..nbytes{
            code |= (self.data[index + i as usize] as u32) << (i*8);
        }
        return Ok(code);
    }
    pub fn store(&mut self,addr:u32,size:u32,value:u32) -> Result<(),Exception>{
        if ![8,16,32].contains(&size){
            return Err(Exception::LoadAccessFault(addr));
        }
        let index = (addr - DRAM_BASE) as usize;
        let nbytes = size / 8;
        for i in 0..nbytes{
            self.data[index + i as usize] = ((value >> (i*8)) & 0xff) as u8;
        }
        Ok(())
    }

}