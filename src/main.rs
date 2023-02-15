mod cpu;
mod display;
mod font;

use cpu::Cpu;
use std::{thread, time};

fn main() {
    let cpu = Cpu::new();
    let keypad = [true; 16];
    // TODO: poll keyboard

    let outputState = cpu.cycle(keypad);

    //TODO: update display
    

    thread::sleep(time::Duration::from_millis(200));
 
    
}
