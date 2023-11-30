mod cpu;
mod bus;
mod dram;
mod exceptions;
mod param;
mod csr;
use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_binary("/Users/zhangximing/remu/complex.bin");
    cpu.run();
}
