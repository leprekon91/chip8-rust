use rand;
use rand::Rng;

use crate::font;
use font::FONT_SET;

const MEMORY_SIZE: usize = 4096;

const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_WIDTH: usize = 64;

const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const OPCODE_SIZE: usize = 2;

const PROGRAM_START: usize = 0x200;

pub struct Cpu {
    memory: [u8; MEMORY_SIZE],
    v_registers: [u8; REGISTER_COUNT], // V0 - VF
    index_register: usize,
    program_counter: usize,
    stack: [usize; 16],
    stack_pointer: usize,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    display_changed: bool,
}

/**
 * @brief Enum to represent the program counter logic
 */
enum PcInstructions {
    Next,
    Skip,
    Jump(usize),
}

pub struct OutputState {
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    display_changed: bool,
    beep: bool,
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
            program_counter: PROGRAM_START as usize,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT], // 64x32 display init to 0 (clear)
            display_changed: false,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            if i >= MEMORY_SIZE - PROGRAM_START {
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

    fn exec_opcode(&mut self, opcode: u16) -> PcInstructions {
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
            _ => PcInstructions::Next,
        };

        pc_change
    }

    /**
     * OPCODES - Instruction Implementations
     */

    // CLS: Clear the display.
    fn op_00e0(&mut self) -> PcInstructions {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.display[y][x] = 0;
            }
        }
        self.display_changed = true;
        PcInstructions::Next
    }

    // RET: Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn op_00ee(&mut self) -> PcInstructions {
        self.stack_pointer -= 1;
        PcInstructions::Jump(self.stack[self.stack_pointer])
    }

    // JP addr: Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn op_1nnn(&mut self, nnn: usize) -> PcInstructions {
        PcInstructions::Jump(nnn.into())
    }

    // CALL addr: Call subroutine at nnn.
    // The interpreter pushes the current PC to the stack. The PC is then set to nnn.
    fn op_2nnn(&mut self, nnn: usize) -> PcInstructions {
        self.stack[self.stack_pointer] = self.program_counter + (OPCODE_SIZE);
        self.stack_pointer += 1;
        PcInstructions::Jump(nnn.into())
    }

    // SE Vx, byte: Skip next instruction if registers[x] = kk.
    // The interpreter compares register registers[x] to kk, and if they are equal,
    // increments the program counter by 2 (i.e. skips the next instruction).
    fn op_3xkk(&mut self, x: usize, kk: u8) -> PcInstructions {
        if self.v_registers[x] == kk {
            PcInstructions::Skip
        } else {
            PcInstructions::Next
        }
    }

    // SNE Vx, byte: Skip next instruction if registers[x] != kk.
    // The interpreter compares register registers[x] to kk, and if they are not equal,
    // increments the program counter by 2.
    fn op_4xkk(&mut self, x: usize, kk: u8) -> PcInstructions {
        if self.v_registers[x] != kk {
            PcInstructions::Skip
        } else {
            PcInstructions::Next
        }
    }

    // SE Vx, Vy: Skip next instruction if registers[x] = registers[y].
    // The interpreter compares register registers[x] to register registers[y], and if they are equal,
    // increments the program counter by 2.
    fn op_5xy0(&mut self, x: usize, y: usize) -> PcInstructions {
        if self.v_registers[x] == self.v_registers[y] {
            PcInstructions::Skip
        } else {
            PcInstructions::Next
        }
    }

    // LD Vx, byte: Set registers[x] = kk.
    // The interpreter puts the value kk into register registers[x].
    fn op_6xkk(&mut self, x: usize, kk: u8) -> PcInstructions {
        self.v_registers[x] = kk;
        PcInstructions::Next
    }

    // ADD Vx, byte: Set registers[x] = registers[x] + kk.
    // Adds the value kk to the value of register registers[x], then stores the result in registers[x].
    fn op_7xkk(&mut self, x: usize, kk: u8) -> PcInstructions {
        // wrappping add: 255 + 1 = 0, prevent panic when register overflows
        self.v_registers[x] = self.v_registers[x].wrapping_add(kk);
        PcInstructions::Next
    }

    // LD Vx, Vy: Set registers[x] = registers[y].
    // Stores the value of register registers[y] in register registers[x].
    fn op_8xy0(&mut self, x: usize, y: usize) -> PcInstructions {
        self.v_registers[x] = self.v_registers[y];
        PcInstructions::Next
    }

    // OR Vx, Vy: Set registers[x] = registers[x] OR registers[y].
    // Performs a bitwise OR on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy1(&mut self, x: usize, y: usize) -> PcInstructions {
        self.v_registers[x] |= self.v_registers[y];
        PcInstructions::Next
    }

    // AND Vx, Vy: Set registers[x] = registers[x] AND registers[y].
    // Performs a bitwise AND on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy2(&mut self, x: usize, y: usize) -> PcInstructions {
        self.v_registers[x] &= self.v_registers[y];
        PcInstructions::Next
    }

    // XOR Vx, Vy: Set registers[x] = registers[x] XOR registers[y].
    // Performs a bitwise exclusive OR on the values of registers[x] and registers[y],
    // then stores the result in registers[x].
    fn op_8xy3(&mut self, x: usize, y: usize) -> PcInstructions {
        self.v_registers[x] ^= self.v_registers[y];
        PcInstructions::Next
    }

    // ADD Vx, Vy: Set registers[x] = registers[x] + registers[y], set VF = carry.
    // The values of registers[x] and registers[y] are added together.

    fn op_8xy4(&mut self, x: usize, y: usize) -> PcInstructions {
        // If the result is greater than 8 bits (i.e., > 255,) registers[x] is set to the lowest 8 bits of the result,
        // and VF is set to 1, otherwise 0.
        let (result, overflow) = self.v_registers[x].overflowing_add(self.v_registers[y]);
        self.v_registers[x] = result;

        self.v_registers[0xF] = if overflow { 1 } else { 0 };

        PcInstructions::Next
    }

    // SUB Vx, Vy: Set registers[x] = registers[x] - registers[y], set VF = NOT borrow.
    // registers[y] is subtracted from registers[x], and the results stored in registers[x].
    fn op_8xy5(&mut self, x: usize, y: usize) -> PcInstructions {
        // If registers[x] > registers[y], then VF is set to 1, otherwise 0.
        self.v_registers[0xF] = if self.v_registers[x] > self.v_registers[y] {
            1
        } else {
            0
        };
        self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);

        PcInstructions::Next
    }

    // SHR Vx {, Vy}: Set registers[x] = registers[x] SHR 1. (Shift Right)
    // If the least-significant bit of registers[x] is 1, then VF is set to 1, otherwise 0.
    // Then registers[x] is divided by 2.
    fn op_8x06(&mut self, x: usize) -> PcInstructions {
        self.v_registers[0xF] = self.v_registers[x] & 0x1;
        self.v_registers[x] >>= 1;

        PcInstructions::Next
    }

    // SUBN Vx, Vy: Set registers[x] = registers[y] - registers[x], set VF = NOT borrow.
    // If registers[y] > registers[x], then VF is set to 1, otherwise 0.
    // Then registers[x] is subtracted from registers[y], and the results stored in registers[x].
    fn op_8xy7(&mut self, x: usize, y: usize) -> PcInstructions {
        self.v_registers[0xF] = if self.v_registers[y] > self.v_registers[x] {
            1
        } else {
            0
        };
        self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);

        PcInstructions::Next
    }

    // SHL Vx {, Vy}: Set registers[x] = registers[x] SHL 1. (Shift Left)
    // If the most-significant bit of registers[x] is 1, then VF is set to 1, otherwise to 0.
    // Then registers[x] is multiplied by 2.
    fn op_8x0e(&mut self, x: usize) -> PcInstructions {
        self.v_registers[0xF] = self.v_registers[x] >> 7;
        self.v_registers[x] <<= 1;

        PcInstructions::Next
    }

    // SNE Vx, Vy: Skip next instruction if registers[x] != registers[y].
    // The values of registers[x] and registers[y] are compared, and if they are not equal,
    // the program counter is increased by 2.
    fn op_9xy0(&mut self, x: usize, y: usize) -> PcInstructions {
        if self.v_registers[x] != self.v_registers[y] {
            PcInstructions::Skip
        } else {
            PcInstructions::Next
        }
    }

    // LD I, addr: Set I = nnn.
    // The value of Index register is set to nnn.
    fn op_annn(&mut self, nnn: usize) -> PcInstructions {
        self.index_register = nnn;
        PcInstructions::Next
    }

    // JP V0, addr: Jump to location nnn + registers[0].
    // The program counter is set to nnn plus the value of registers[0].
    fn op_bnnn(&mut self, nnn: usize) -> PcInstructions {
        let addr = nnn + self.v_registers[0] as usize;
        PcInstructions::Jump(addr.into())
    }

    // RND Vx, byte: Set registers[x] = random byte AND kk.
    // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
    // The results are stored in registers[x].
    fn op_cxkk(&mut self, x: usize, kk: u8) -> PcInstructions {
        let mut rng = rand::thread_rng();
        self.v_registers[x] = rng.gen::<u8>() & kk;
        PcInstructions::Next
    }

    // DRW Vx, Vy, n
    // The interpreter reads n bytes from memory, starting at the address
    // stored in Index Register. These bytes are then displayed as sprites on screen at
    // coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise
    // it is set to 0. If the sprite is positioned so part of it is outside
    // the coordinates of the display, it wraps around to the opposite side
    // of the screen.
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) -> PcInstructions {
        self.v_registers[0x0f] = 0;
        for byte in 0..n {
            let y = (self.v_registers[y] as usize + byte) % DISPLAY_HEIGHT;
            for bit in 0..8 {
                let x = (self.v_registers[x] as usize + bit) % DISPLAY_WIDTH;
                let color = (self.memory[self.index_register + byte] >> (7 - bit)) & 1;
                self.v_registers[0x0f] |= color & self.display[y][x];
                self.display[y][x] ^= color;
            }
        }

        self.display_changed = true;
        PcInstructions::Next
    }

    // SKP Vx: Skip next instruction if key with the value of registers[x] is pressed.
    // Checks the keyboard, and if the key corresponding to the value of registers[x] is currently in the down position,
    // PC is increased by 2.
    fn op_ex9e(&mut self, x: usize) -> PcInstructions {
        PcInstructions::skip_if(self.keypad[self.v_registers[x] as usize])
    }

    // SKNP Vx: Skip next instruction if key with the value of registers[x] is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of registers[x] is currently in the up position,
    // PC is increased by 2.
    fn op_exa1(&mut self, x: usize) -> PcInstructions {
        PcInstructions::skip_if(!self.keypad[self.v_registers[x] as usize])
    }

    // LD Vx, DT: Set registers[x] = delay_timer.
    // The value of DT is placed into registers[x].
    fn op_fx07(&mut self, x: usize) -> PcInstructions {
        self.v_registers[x] = self.delay_timer;
        PcInstructions::Next
    }

    // LD Vx, K: Wait for a key press, store the value of the key in registers[x].
    // All execution stops until a key is pressed, then the value of that key is stored in registers[x].
    fn op_fx0a(&mut self, x: usize) -> PcInstructions {
        self.keypad_waiting = true;
        self.keypad_register = x;
        PcInstructions::Next
    }

    // LD DT, Vx: Set delay_timer = registers[x].
    // DT is set equal to the value of registers[x].
    fn op_fx15(&mut self, x: usize) -> PcInstructions {
        self.delay_timer = self.v_registers[x];
        PcInstructions::Next
    }

    // LD ST, Vx: Set sound_timer = registers[x].
    // ST is set equal to the value of registers[x].
    fn op_fx18(&mut self, x: usize) -> PcInstructions {
        self.sound_timer = self.v_registers[x];
        PcInstructions::Next
    }

    // ADD I, Vx: Set I = I + registers[x].
    // The values of registers[x] and Index Register are added, and the results are stored in Index Register.
    fn op_fx1e(&mut self, x: usize) -> PcInstructions {
        self.index_register += self.v_registers[x] as usize;
        self.v_registers[0x0f] = if self.index_register > 0x0F00 { 1 } else { 0 };
        PcInstructions::Next
    }

    // LD F, Vx: Set I = location of sprite for digit registers[x].
    // The value of registers[x] is used as the index into the font set.
    // The value of Index Register is set to the location for the hexadecimal sprite corresponding to the value of registers[x].
    fn op_fx29(&mut self, x: usize) -> PcInstructions {
        self.index_register = (self.v_registers[x] as usize) * 5;
        PcInstructions::Next
    }

    // LD B, Vx: Store BCD representation of registers[x] in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of registers[x], and places the hundreds digit in memory at location in Index Register,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33(&mut self, x: usize) -> PcInstructions {
        let value = self.v_registers[x];
        self.memory[self.index_register as usize] = value / 100;
        self.memory[self.index_register as usize + 1] = (value / 10) % 10;
        self.memory[self.index_register as usize + 2] = (value % 100) % 10;
        PcInstructions::Next
    }

    // LD [I], Vx: Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through registers[x] into memory, starting at the address in Index Register.
    fn op_fx55(&mut self, x: usize) -> PcInstructions {
        for i in 0..=x {
            self.memory[self.index_register as usize + i] = self.v_registers[i];
        }
        PcInstructions::Next
    }

    // LD Vx, [I]: Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through registers[x].
    fn op_fx65(&mut self, x: usize) -> PcInstructions {
        for i in 0..=x {
            self.v_registers[i] = self.memory[self.index_register as usize + i];
        }
        PcInstructions::Next
    }

    // MAIN LOOP
    pub fn cycle(&mut self, keypad: [bool; 16]) -> OutputState {
        self.keypad = keypad;
        self.display_changed = false;

        if self.keypad_waiting {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.v_registers[self.keypad_register] = i as u8;
                    break;
                }
            }
        } else {
            // Fetch Opcode
            let opcode = self.fetch_opcode();

            // Run Opcode instruction
            let pc_instruction = self.exec_opcode(opcode);

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
                self.sound_timer -= 1;
            }
        }

        let display = self.display.clone();

        // Render Display
        OutputState {
            display: display,
            display_changed: self.display_changed,
            beep: self.sound_timer > 0,
        }
    }
}
