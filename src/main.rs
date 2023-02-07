mod display;

use display::*;
use std::{thread, time};

fn main() {
    let mut display = Display::new(64, 32);
    
    for i in 0..64 {
        display.toggle_pixel(i, 10);
        display.render();

        thread::sleep(time::Duration::from_millis(200));
    }
}
