use std::sync::{
    atomic::{AtomicBool,Ordering},
    Arc,Condvar,Mutex
};
use std::io::prelude::*;
use std::io;
use std::thread;
use crate::param::*;
use crate::exceptions::Exception;

const UART_SIZE: u32 = 8;
const UART_LSR: u32 = 5;
const MASK_UART_LSR_RX : u8 = 1;
const MASK_UART_LSR_TX : u8 = 0x20;
const UART_IRQ : u32 = 10;
const UART_RHR : u32 = 0;
const UART_THR : u32 = 0;
const UART_LCR : u32 = 3;

pub struct UartController{
    uart_regs:Arc<(Mutex<[u8;UART_SIZE as usize]>,Condvar)>,
    interrupt:Arc<AtomicBool>
}

impl UartController{
    pub fn new() -> Self{
        let mut arr = [0; UART_SIZE as usize];
        // set send buffer empty
        arr[UART_LSR as usize] |= MASK_UART_LSR_TX;
        let uart_regs = Arc::new((Mutex::new(arr),Condvar::new()));
        let interrupt = Arc::new(AtomicBool::new(false));

        let read_uart = Arc::clone(&uart_regs);
        let read_interrupt = Arc::clone(&interrupt);
        let mut byte = [0];
        thread::spawn(move ||{
            loop{
                match io::stdin().read(&mut byte){
                    Ok(_) =>{
                        let (uart_regs,cvar) = &*read_uart;
                        let mut array = uart_regs.lock().unwrap();
                        while (array[UART_LSR as usize] & MASK_UART_LSR_RX) == 1{
                            array = cvar.wait(array).unwrap();
                        }
                        array[UART_RHR as usize] = byte[0];
                        read_interrupt.store(true, Ordering::Release);
                        array[UART_LSR as usize] |= MASK_UART_LSR_RX;
                    }
                    Err(e) => println!("{}",e)

                }
            }

        });
        Self{uart_regs,interrupt}
    }
    pub fn load(&self,addr:u32) -> Result<u32,Exception>{
        let offset = addr - UART_BASE;
        let (arr,cond) = &*self.uart_regs;
        let mut array = arr.lock().unwrap();
        match offset{
            UART_RHR => {
                cond.notify_one();
                array[UART_LSR as usize] &= !MASK_UART_LSR_RX;
                Ok(array[UART_RHR as usize] as u32)
            }
            _ => {
                Ok(array[offset as usize] as u32)
            }
        }

    }
    pub fn store(&mut self,addr: u32,data:u32) {
        let offset = addr - UART_BASE;
        let (arr,cond) = &*self.uart_regs;
        let mut array = arr.lock().unwrap();
        match offset{
            UART_THR => {
                print!("{}",data as u8 as char);
                io::stdout().flush().unwrap();
            }
            _ => {
                array[offset as usize] = data as u8;
            }
        }
    }
    pub fn is_iterrupt(&self) -> bool{
        self.interrupt.swap(false, Ordering::Acquire)
    }
}