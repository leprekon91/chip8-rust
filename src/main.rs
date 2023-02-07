mod cpu;

use cpu::Cpu;
use std::{thread, time};

fn main() {
    let cpu = Cpu::new();
    let keypad = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
    cpu.cycle(keypad);

    thread::sleep(time::Duration::from_millis(200));
 
    
}
