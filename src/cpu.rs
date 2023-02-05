use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};

// use crate::display::Display;
// use crate::keypad::Keypad;

static FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn not_implemented(op: usize, pc: usize) {
    println!("Not implemented:: op: {:x}, pc: {:x}", op, pc)
}

pub struct Cpu {
    program: usize, // Program counter
    opcode: u16,    // Current opcode

    stack: [u16; 16],     // Stack
    stack_pointer: usize, // Stack pointer

    v: [u8; 16], // Registers
    i: u16,      // Index register

    memory: [u8; 4096], // Memory

    pub keypad: Keypad,
    pub display: Display,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            program: 0x200, // Program counter starts at 0x200
            opcode: 0,      // Current opcode defaults to 0

            stack: [0; 16],   // empty Stack
            stack_pointer: 0, // Stack pointer defaults to 0, beginning of stack

            v: [0; 16], // Registers
            i: 0,       // Index register defaults to 0

            memory: [0; 4096], // empty Memory

            keypad: Keypad::new(),   // Keypad
            display: Display::new(), // Display
        };

        // Load font
        for i in 0..80 {
            cpu.memory[i] = FONTS[i];
        }

        cpu
    }

    pub fn load_program(&mut self, path: &str) {
        // try to open the file
        let mut reader = File::open(path).expect("Could not open file");

        // declare a new buffer
        let mut buffer = Vec::new();

        // read the file into a buffer
        reader
            .read_to_end(&mut buffer)
            .expect("Could not read file");

        for (i, byte) in buffer.iter().enumerate() {
            self.memory[i + 0x200] = *byte;
        }
    }

    pub fn get_opcode(&mut self) {
        // Get the opcode from the memory

        // Shift the first byte left by 8 bits
        // OR switch the second byte with the first byte
        self.opcode =
            (self.memory[self.program] as u16) << 8 | self.memory[self.program + 1] as u16;
        thread::sleep(time::Duration::from_micros(500));
    }

    pub fn decode_opcode(&mut self) {}

    fn op_x(&self) -> usize {
        ((self.opcode & 0x0F00) >> 8) as usize
    }
    fn op_y(&self) -> usize {
        ((self.opcode & 0x00F0) >> 4) as usize
    }
    fn op_n(&self) -> u8 {
        (self.opcode & 0x000F) as u8
    }
    fn op_nn(&self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }
    fn op_nnn(&self) -> u16 {
        self.opcode & 0x0FFF
    }

    pub fn cycle(&mut self) {
        self.get_opcode();
        self.decode_opcode();
    }
}
