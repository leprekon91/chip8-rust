/**
 * @file display.rs
 * @brief Display module to draw whatever is in memmory to the CLI
 */
use std::io::{self, Write};

pub struct Display {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u8>,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Display {
        Display {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0;
        }
    }

    pub fn toggle_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = (x + (y * self.width)) % self.buffer.len();
        let old_pixel = self.buffer[index];
        self.buffer[index] ^= 1;
        old_pixel == 1
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (i, row) in sprite.iter().enumerate() {
            for j in 0..8 {
                let pixel = (row >> (7 - j)) & 0x1;
                let index = (x + j + ((y + i) * self.width)) % self.buffer.len();
                if pixel == 1 && self.buffer[index] == 1 {
                    collision = true;
                }
                self.buffer[index] ^= pixel;
            }
        }
        collision
    }

    pub fn render(&self) {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        for i in 0..self.buffer.len() {
            if i % self.width == 0 {
                println!();
            }
            if self.buffer[i] == 1 {
                print!("█");
            } else {
                print!("░");
            }
        }
        
    }
}
