mod cpu;
mod display;
mod font;

use cpu::Cpu;
use std::{thread, time};

fn main() {
    let mut cpu =  Cpu::new();
    let keypad = [true; 16];
   // TODO: load rom 
    // TODO: poll keyboard

    let outputState = cpu.cycle(keypad);

    //TODO: update display
    

    thread::sleep(time::Duration::from_millis(200));
 
    
}
