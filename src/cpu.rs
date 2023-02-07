use rand;
use rand::Rng;
use std::{thread, time};

use display::Display;
use font::FONT_SET;
use stack::Stack;

const MEMORY_SIZE: usize = 4096;

const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;

const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const OPCODE_SIZE: usize = 2;

const PROGRAM_START: usize = 0x200;

const CLOCK_SPEED: u64 = 500;

pub struct Cpu {
    pub memory: [u8; MEMORY_SIZE],
    pub v_registers: [u8; REGISTER_COUNT], // V0 - VF
    pub index_register: u16,
    pub program_counter: u16,
    pub stack: Stack<u16>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [bool; 16],
    pub display: Display,
}

/**
 * @brief Enum to represent the program counter logic
 */
enum PcInstructions {
    Next,
    Skip,
    Jump(usize),
}

impl PcInstructions {
    // Helper function to skip the next instruction if a condition is true
    fn skip_if(condition: bool) -> PcInstructions {
        if condition {
            PcInstructions::Skip
        } else {
            PcInstructions::Next
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        // Load Font Set
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        for (i, &byte) in FONT_SET.iter().enumerate() {
            memory[i] = byte;
        }

        // Initialize CPU registers and memory
        Cpu {
            memory,
            v_registers: [0; REGISTER_COUNT], // V0 - VF init to 0
            index_register: 0,
            program_counter: PROGRAM_START as u16,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            display: Display::new(DISPLAY_WIDTH, DISPLAY_HEIGHT), // 64x32 display init to 0 (clear)
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            if (i >= MEMORY_SIZE - PROGRAM_START) {
                panic!("Program too large to fit in memory");
            }

            self.memory[PROGRAM_START + i] = byte;
        }
    }

    fn fetch_opcode(&self) -> u16 {
        //each opcode is 2 bytes long, PC points to the first one
        let first_byte = self.memory[self.program_counter as usize] as u16;
        let second_byte = self.memory[(self.program_counter + 1) as usize] as u16;

        // return the two bytes as a single opcode of 2 words
        return (first_byte << 8) | second_byte;
    }

    fn exec_opcode(&self, opcode: u16) -> PcInstructions {
        // nibbles = HEX Digits of the opcode
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        // break apart parameters o the instruction
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        // match to instruction, if no match,go to next byte in the program
        let pc_change = match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8x06(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op_8x0e(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0a, _, _, _) => self.op_annn(nnn),
            (0x0b, _, _, _) => self.op_bnnn(nnn),
            (0x0c, _, _, _) => self.op_cxkk(x, kk),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x),
            _ => ProgramCounter::Next,
        };

        pc_change
    }

    /**
     * OPCODES - Instruction Implementations
     */

    // CLS: Clear the display.
    fn op_00e0(&self) -> PcInstructions {
        self.display.clear();
        ProgramCounter::Next
    }

    // RET: Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn op_00ee(&mut self) -> ProgramCounter {
        let addr = self.stack.pop();
        ProgramCounter::Jump(addr)
    }

    // JP addr: Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Jump(nnn)
    }

    // CALL addr: Call subroutine at nnn.
    // The interpreter pushes the current PC to the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, nnn: usize) -> ProgramCounter {
        self.stack.push(self.program_counter);
        ProgramCounter::Jump(nnn)
    }

    // SE Vx, byte: Skip next instruction if registers[x] = kk.
    // The interpreter compares register registers[x] to kk, and if they are equal,
    // increments the program counter by 2 (i.e. skips the next instruction).
    fn op_3xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.registers[x] == kk {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // SNE Vx, byte: Skip next instruction if registers[x] != kk.
    // The interpreter compares register registers[x] to kk, and if they are not equal,
    // increments the program counter by 2.
    fn op_4xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        if self.registers[x] != kk {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // SE Vx, Vy: Skip next instruction if registers[x] = registers[y].
    // The interpreter compares register registers[x] to register registers[y], and if they are equal,
    // increments the program counter by 2.
    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.registers[x] == self.registers[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // LD Vx, byte: Set registers[x] = kk.
    // The interpreter puts the value kk into register registers[x].
    fn op_6xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        self.registers[x] = kk;
        ProgramCounter::Next
    }

    // ADD Vx, byte: Set registers[x] = registers[x] + kk.
    // Adds the value kk to the value of register registers[x], then stores the result in registers[x].
    fn op_7xkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        // wrappping add: 255 + 1 = 0, prevent panic when register overflows
        self.registers[x] = self.registers[x].wrapping_add(kk);
        ProgramCounter::Next
    }

    // LD Vx, Vy: Set registers[x] = registers[y].
    // Stores the value of register registers[y] in register registers[x].
    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] = self.registers[y];
        ProgramCounter::Next
    }

    // OR Vx, Vy: Set registers[x] = registers[x] OR registers[y].
    // Performs a bitwise OR on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] |= self.registers[y];
        ProgramCounter::Next
    }

    // AND Vx, Vy: Set registers[x] = registers[x] AND registers[y].
    // Performs a bitwise AND on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] &= self.registers[y];
        ProgramCounter::Next
    }

    // XOR Vx, Vy: Set registers[x] = registers[x] XOR registers[y].
    // Performs a bitwise exclusive OR on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[x] ^= self.registers[y];
        ProgramCounter::Next
    }

    // ADD Vx, Vy: Set registers[x] = registers[x] + registers[y], set VF = carry.
    // The values of registers[x] and registers[y] are added together.

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounter {
        // If the result is greater than 8 bits (i.e., > 255,) registers[x] is set to the lowest 8 bits of the result,
        // and VF is set to 1, otherwise 0.
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;

        self.registers[0xF] = if overflow { 1 } else { 0 };

        ProgramCounter::Next
    }

    // SUB Vx, Vy: Set registers[x] = registers[x] - registers[y], set VF = NOT borrow.
    // registers[y] is subtracted from registers[x], and the results stored in registers[x].
    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounter {
        // If registers[x] > registers[y], then VF is set to 1, otherwise 0.
        self.registers[0xF] = if self.registers[x] > self.registers[y] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);

        ProgramCounter::Next
    }

    // SHR Vx {, Vy}: Set registers[x] = registers[x] SHR 1. (Shift Right)
    // If the least-significant bit of registers[x] is 1, then VF is set to 1, otherwise 0.
    // Then registers[x] is divided by 2.
    fn op_8xy6(&mut self, x: usize, _y: usize) -> ProgramCounter {
        self.registers[0xF] = self.registers[x] & 0x1;
        self.registers[x] >>= 1;

        ProgramCounter::Next
    }

    // SUBN Vx, Vy: Set registers[x] = registers[y] - registers[x], set VF = NOT borrow.
    // If registers[y] > registers[x], then VF is set to 1, otherwise 0.
    // Then registers[x] is subtracted from registers[y], and the results stored in registers[x].
    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounter {
        self.registers[0xF] = if self.registers[y] > self.registers[x] {
            1
        } else {
            0
        };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);

        ProgramCounter::Next
    }

    // SHL Vx {, Vy}: Set registers[x] = registers[x] SHL 1. (Shift Left)
    // If the most-significant bit of registers[x] is 1, then VF is set to 1, otherwise to 0.
    // Then registers[x] is multiplied by 2.
    fn op_8xye(&mut self, x: usize, _y: usize) -> ProgramCounter {
        self.registers[0xF] = self.registers[x] >> 7;
        self.registers[x] <<= 1;

        ProgramCounter::Next
    }

    // SNE Vx, Vy: Skip next instruction if registers[x] != registers[y].
    // The values of registers[x] and registers[y] are compared, and if they are not equal,
    // the program counter is increased by 2.
    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounter {
        if self.registers[x] != self.registers[y] {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // LD I, addr: Set I = nnn.
    // The value of Index register is set to nnn.
    fn op_annn(&mut self, nnn: u16) -> ProgramCounter {
        self.index_register = nnn;
        ProgramCounter::Next
    }

    // JP V0, addr: Jump to location nnn + registers[0].
    // The program counter is set to nnn plus the value of registers[0].
    fn op_bnnn(&mut self, nnn: u16) -> ProgramCounter {
        ProgramCounter::Jump(nnn + self.registers[0] as u16)
    }

    // RND Vx, byte: Set registers[x] = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    // The results are stored in registers[x].
    fn op_cxkk(&mut self, x: usize, kk: u8) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.registers[x] = rng.gen::<u8>() & kk;
        ProgramCounter::Next
    }

    // DRW Vx, Vy, n
    // The interpreter reads n bytes from memory, starting at the address
    // stored in Index Register. These bytes are then displayed as sprites on screen at
    // coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise
    // it is set to 0. If the sprite is positioned so part of it is outside
    // the coordinates of the display, it wraps around to the opposite side
    // of the screen.
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        self.registers[0xF] = 0;
        for byte in 0..n {
            let sprite_byte = self.memory[self.index_register as usize + byte];
            for bit in 0..8 {
                let x_coord = (self.registers[x] as usize + bit) % DISPLAY_WIDTH;
                let y_coord = (self.registers[y] as usize + byte) % DISPLAY_HEIGHT;
                let pixel = self.display.get_pixel(x_coord, y_coord);
                let sprite_pixel = sprite_byte & (0x80 >> bit);
                if sprite_pixel != 0 {
                    if pixel == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.display.set_pixel(x_coord, y_coord, pixel ^ 1);
                }
            }
        }
    }
   
    // SKP Vx: Skip next instruction if key with the value of registers[x] is pressed.
    // Checks the keyboard, and if the key corresponding to the value of registers[x] is currently in the down position,
    // PC is increased by 2.
    fn op_ex9e(&mut self, x: usize) -> ProgramCounter {
        if self.keyboard.is_key_pressed(self.registers[x]) {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // SKNP Vx: Skip next instruction if key with the value of registers[x] is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of registers[x] is currently in the up position,
    // PC is increased by 2.
    fn op_exa1(&mut self, x: usize) -> ProgramCounter {
        if !self.keyboard.is_key_pressed(self.registers[x]) {
            ProgramCounter::Skip
        } else {
            ProgramCounter::Next
        }
    }

    // LD Vx, DT: Set registers[x] = delay_timer.
    // The value of DT is placed into registers[x].
    fn op_fx07(&mut self, x: usize) -> ProgramCounter {
        self.registers[x] = self.delay_timer;
        ProgramCounter::Next
    }

    // LD Vx, K: Wait for a key press, store the value of the key in registers[x].
    // All execution stops until a key is pressed, then the value of that key is stored in registers[x].
    fn op_fx0a(&mut self, x: usize) -> ProgramCounter {
        if let Some(key) = self.keyboard.wait_for_key() {
            self.registers[x] = key;
            ProgramCounter::Next
        } else {
            ProgramCounter::Skip
        }
    }

    // LD DT, Vx: Set delay_timer = registers[x].
    // DT is set equal to the value of registers[x].
    fn op_fx15(&mut self, x: usize) -> ProgramCounter {
        self.delay_timer = self.registers[x];
        ProgramCounter::Next
    }

    // LD ST, Vx: Set sound_timer = registers[x].
    // ST is set equal to the value of registers[x].
    fn op_fx18(&mut self, x: usize) -> ProgramCounter {
        self.sound_timer = self.registers[x];
        ProgramCounter::Next
    }

    // ADD I, Vx: Set I = I + registers[x].
    // The values of registers[x] and Index Register are added, and the results are stored in Index Register.
    fn op_fx1e(&mut self, x: usize) -> ProgramCounter {
        self.index_register += self.registers[x] as u16;
        ProgramCounter::Next
    }

    // LD F, Vx: Set I = location of sprite for digit registers[x].
    // The value of registers[x] is used as the index into the font set.
    // The value of Index Register is set to the location for the hexadecimal sprite corresponding to the value of registers[x].
    fn op_fx29(&mut self, x: usize) -> ProgramCounter {
        self.index_register = (self.registers[x] as u16) * 5;
        ProgramCounter::Next
    }

    // LD B, Vx: Store BCD representation of registers[x] in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of registers[x], and places the hundreds digit in memory at location in Index Register,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, x: usize) -> ProgramCounter {
        let value = self.registers[x];
        self.memory[self.index_register as usize] = value / 100;
        self.memory[self.index_register as usize + 1] = (value / 10) % 10;
        self.memory[self.index_register as usize + 2] = (value % 100) % 10;
        ProgramCounter::Next
    }

    // LD [I], Vx: Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through registers[x] into memory, starting at the address in Index Register.
    fn op_fx55(&mut self, x: usize) -> ProgramCounter {
        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.registers[i];
        }
        ProgramCounter::Next
    }

    // LD Vx, [I]: Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through registers[x].
    fn op_fx65(&mut self, x: usize) -> ProgramCounter {
        for i in 0..=x {
            self.registers[i] = self.memory[self.index_register as usize + i];
        }
        ProgramCounter::Next
    }


    // MAIN LOOP
    pub fn cycle(&mut self) {
        // Fetch Opcode
        let opcode = fetch_opcode();

        // Run Opcode instruction
        let pc_instruction = decode_opcode(opcode);

        // Update Program Counter
        match pc_instruction {
            PcInstructions::Next => self.program_counter += OPCODE_SIZE,
            PcInstructions::Skip => self.program_counter += 2 * OPCODE_SIZE,
            PcInstructions::Jump(addr) => self.program_counter = addr,
        }

        // Update Timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // println!("BEEP!");
            }
            self.sound_timer -= 1;
        }

        // Render Display
        self.display.render();

        // Delay to slow down the CPU
        thread::sleep(time::Duration::from_micros(CLOCK_SPEED));
    }
}
