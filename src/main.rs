mod cpu;
mod bus;
mod dram;
mod exceptions;
mod param;
mod csr;
mod interrupt;
mod uart;
use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_binary("/Users/zhangximing/Downloads/complex.bin");
    cpu.run();
}
