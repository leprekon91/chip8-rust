use crate::cpu::Cpu;



fn not_implemented(op: usize, pc: usize) {
    println!("Not implemented:: op: {:x}, pc: {:x}", op, pc)
}

fn op_0xxx(cpu: &mut Cpu) {
    let opcode = cpu.opcode;
    match opcode & 0x000f {
        0x0000 => {
            // 0: Clears the screen.
            cpu.display.clear();
        }
        0x000e => {
            // E: Returns from a subroutine.
            cpu.stack_pointer -= 1;
            cpu.program = cpu.stack[cpu.stack_pointer] as usize;
        }
        _ => {
            not_implemented(opcode as usize, cpu.program);
        }
    }
}

fn op_1xxx(cpu: &mut Cpu) {
    let opcode = cpu.opcode;
    // 1NNN: Jumps to address NNN.
    cpu.program = (opcode & 0x0fff) as usize;
}

fn op_2xxx(cpu: &mut Cpu) {
    cpu.stack[cpu.stack_pointer] = cpu.program as u16;
    cpu.stack_pointer += 1;
    cpu.program = cpu.op_nnn() as usize;
}

fn decode_opcode(cpu: Cpu) {
   let opcode = cpu.opcode;

  match opcode &0xf000{
    0x0000 => op_0xxx(cpu),
    _ => not_implemented(opcode as usize, cpu.program),
  } 
}
