mod cpu;

use cpu::Cpu;
use std::{thread, time};

fn main() {
    let cpu = Cpu::new();
    let keypad = [true; 16];
    cpu.cycle(keypad);

    thread::sleep(time::Duration::from_millis(200));
 
    
}
