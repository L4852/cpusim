use std::{thread, time::Duration};

struct Processor {
    registers: [i32; 4],
    program_counter: usize,
    ram: [i32; 64],
    flag_register: i32,
    halt: bool
}

// ____________     0000      00000000000000000
//                   4               18
//    EXTRA       OPCODE            DATA

fn print_as_assembly(instruction: i32) {
    let opcode = instruction >> 18;
    let operand = instruction & (!(0b1111 << 18));

    let mut final_string = String::new();

    match opcode {
        0 => { final_string.push_str("NO-OP ")},
        1 => { 
            final_string.push_str("LOAD_IMMED ");

            let immediate_value = operand >> 2;
            let target_register = operand & (!(0b1111 << 2));

            final_string.push_str(&i32::to_string(&immediate_value));
            final_string.push_str(" R");
            final_string.push_str(&i32::to_string(&target_register));
        },
        2 => { final_string.push_str("ADD ")},
        3 => { final_string.push_str("SUB ")},
        4 => { final_string.push_str("CMP_IMMED ")},
        5 => { final_string.push_str("JMP ")},
        6 => { final_string.push_str("JMP_EQ ")},
        7 => { final_string.push_str("JMP_GT ")},
        8 => { final_string.push_str("JMP_LT ")},
        15 => { final_string.push_str("HALT")}
        _ => {}
    }

    println!("{}", final_string);
}

impl Processor {
    fn new() -> Processor {
        Processor {
            registers: [0;4],
            program_counter: 0,
            ram: [0;64],
            flag_register: -1,
            halt: false
        }
    }
    
    fn load_program(&mut self, program:&[i32]) {
        for (i, &instruction) in program.iter().enumerate() {
            self.ram[i] = instruction;
        }
    }

    fn fetch_instruction(&mut self) -> i32 {
        self.ram[self.program_counter]
    }

    fn execute_instruction(&mut self) {
        let instruction = self.fetch_instruction();

        let opcode = instruction >> 18;
        let operand = instruction & (!(0b1111 << 18));

        print_as_assembly(instruction);

        println!();

        println!("OPCODE: {:b}\nOPERAND: {:b}", opcode, operand);

        match opcode {
            1 => {
                let immediate_value = operand >> 2;
                let target_register = operand & (!(0b1111 << 2));
                self.registers[target_register as usize] = immediate_value;

                println!("REG[{}] <- {}", target_register, self.registers[target_register as usize]);
            }
            2 => {
                let reg_a = operand >> 4;
                let reg_b = (operand & 0b001100) >> 2;
                let reg_c = operand & 0b000011;

                self.registers[reg_c as usize] = self.registers[reg_a as usize] + self.registers[reg_b as usize];
 
                println!("REG[{}] <- {}", reg_c, self.registers[reg_c as usize]);
            }
            3 => {
                let reg_a = operand >> 4;
                let reg_b = (operand & 0b001100) >> 2;
                let reg_c = operand & 0b000011;

                self.registers[reg_c as usize] = self.registers[reg_a as usize] - self.registers[reg_b as usize];
 
                println!("REG[{}] <- {}", reg_c, self.registers[reg_c as usize]);
            }
            4 => {
                let immed_compare = operand >> 2; 
                let register_addr = operand & (0b11);

                let result = immed_compare - self.registers[register_addr as usize];

                if result > 0 {
                    self.flag_register = 1;
                }
                else if result == 0 {
                    self.flag_register = 0;
                }
                else if result < 0 {
                    self.flag_register = 2;
                }
                else {
                    self.flag_register = -1;
                }

                println!("CMP -> [{}]", self.flag_register);
            }
            5 => {
                let jump_addr = operand & (0b11111);

                self.program_counter = jump_addr as usize;

                println!("JMP -> [{}]", self.program_counter);
            }
            6 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 0 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = -1;
                }
            }
            7 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 1 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = -1;
                }
            }
            8 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 2 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = -1;
                }
            }
            15 => {
                self.halt = true;
            }
            _ => {
                return;
            }

        }

    }
}


fn assembler(instruction: String) {
    let terms: Vec<&str> = instruction.split_whitespace().collect();

    let mut output_ins = 0;

    for (i, &term) in terms.iter().enumerate() {
        match term {
            "load" => {
                output_ins = 0b0001
            },
            "add" => {
                output_ins = 0b0010
            },
            "sub" => {
                output_ins = 0b0011
            },
            _ => {}
        }
    }



}

fn main() {
    let mut cpu = Processor::new();

    // 0b_0000_000000000000000000

    let program = [
        0b_0001_0000000000000001_01,
        0b_0001_0000000000000001_10,
        0b_0010_000000000000_01_10_10,
        0b_0100_1000000000000000_10,
        0b_0111_0000000000000_00010,
        0b_1111_000000000000000000
    ];

    // for ins in program {
    //     print_as_assembly(ins);
    // }

    // thread::sleep(Duration::from_secs(5));

    cpu.load_program(&program);

    loop {
        println!("[{}]", cpu.program_counter);

        cpu.fetch_instruction();

        cpu.execute_instruction();

        if cpu.program_counter == 63 || cpu.halt {
            break;
        }

        cpu.program_counter += 1;

        println!();

        // thread::sleep(Duration::from_secs(1));
    }
}