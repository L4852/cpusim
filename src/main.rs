use std::fmt::Arguments;
use std::fs::File;
use std::io::{self, Read};
use std::{thread, time::Duration};

struct Processor {
    registers: [u32; 4],
    program_counter: usize,
    ram: [u32; 64],
    flag_register: u32,
    halt: bool,
    debug: bool,
}

fn print_as_assembly(instruction: u32) {
    let opcode = instruction >> 18;

    // Take lower 18 bits
    let operand = instruction & (!(0b1111 << 18));

    let mut final_string = String::new();

    match opcode {
        0 => final_string.push_str("NO-OP "),
        1 => {
            final_string.push_str("LOAD_IMMED ");

            let immediate_value = operand >> 2;
            let target_register = operand & (!(0b1111 << 2));

            final_string.push_str(&u32::to_string(&immediate_value));
            final_string.push_str(" R");
            final_string.push_str(&u32::to_string(&target_register));
        }
        2 => final_string.push_str("ADD "),
        3 => final_string.push_str("SUB "),
        4 => final_string.push_str("CMP_IMMED "),
        5 => final_string.push_str("JMP "),
        6 => final_string.push_str("JMP_EQ "),
        7 => final_string.push_str("JMP_GT "),
        8 => final_string.push_str("JMP_LT "),
        9 => final_string.push_str("STO"),
        15 => final_string.push_str("HALT"),
        _ => {}
    }

    println!("{}", final_string);
}
impl Processor {
    fn new(use_debug: bool) -> Processor {
        Processor {
            registers: [0; 4],
            program_counter: 0,
            ram: [0; 64],
            flag_register: 0,
            halt: false,
            debug: use_debug,
        }
    }

    fn debug_print(&self, text: Arguments) {
        if (self.debug == true) {
            println!("{}", text);
        }
    }

    fn load_program(&mut self, program: &[u32]) {
        for (i, &instruction) in program.iter().enumerate() {
            self.ram[i] = instruction;
        }
    }

    fn fetch_instruction(&mut self) -> u32 {
        self.ram[self.program_counter]
    }

    fn execute_instruction(&mut self) {
        let instruction = self.fetch_instruction();

        let opcode = instruction >> 18;
        let operand = instruction & (!(0b1111 << 18));

        print_as_assembly(instruction);

        println!();

        self.debug_print(format_args!(
            "OPCODE: {:04b}\nOPERAND: {:018b}",
            opcode, operand
        ));

        match opcode {
            1 => {
                let immediate_value = operand >> 2;
                let target_register = operand & (!(0b1111 << 2));
                self.registers[target_register as usize] = immediate_value;

                self.debug_print(format_args!(
                    "\nREG[{}] <- {}",
                    target_register, self.registers[target_register as usize]
                ));
            }
            2 => {
                let reg_a = operand >> 4;
                let reg_b = (operand & 0b001100) >> 2;
                let reg_c = operand & 0b000011;

                self.registers[reg_c as usize] =
                    self.registers[reg_a as usize] + self.registers[reg_b as usize];

                self.debug_print(format_args!(
                    "\nREG[{}] <- {}",
                    reg_c, self.registers[reg_c as usize]
                ));
            }
            3 => {
                let reg_a = operand >> 4;
                let reg_b = (operand & 0b001100) >> 2;
                let reg_c = operand & 0b000011;

                self.registers[reg_c as usize] =
                    self.registers[reg_a as usize] - self.registers[reg_b as usize];

                self.debug_print(format_args!(
                    "\nREG[{}] <- {}",
                    reg_c, self.registers[reg_c as usize]
                ));
            }
            // Compare
            4 => {
                let immed_compare = operand >> 2;
                let register_addr = operand & (0b11);

                // Compare register value to immediate value and save result to flag register

                let result = self.registers[register_addr as usize] as i32 - immed_compare as i32;

                if result > 0 {
                    self.flag_register = 1;
                    self.debug_print(format_args!("\nCMP -> [{}] (GT)", self.flag_register));
                } else if result == 0 {
                    self.flag_register = 2;
                    self.debug_print(format_args!("\nCMP -> [{}] (EQ)", self.flag_register));
                } else if result < 0 {
                    self.flag_register = 3;
                    self.debug_print(format_args!("\nCMP -> [{}] (LT)", self.flag_register));
                } else {
                    self.flag_register = 0;
                }
            }
            // Jump
            5 => {
                let jump_addr = operand & (0b11111);

                self.program_counter = jump_addr as usize;

                self.debug_print(format_args!("\nJMP -> [{}]", self.program_counter));
            }
            // Jump if equal
            6 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 2 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!("\nJEQ -> [{}]", self.program_counter));
            }
            7 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 1 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!("\nJGT -> [{}]", self.program_counter));
            }
            8 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 3 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!("\nJLT -> [{}]", self.program_counter));
            }
            // Store from register to RAM
            9 => {
                let reg_addr = operand & (0b11);
                let ram_addr = (operand & (0b111111 << 2)) >> 2;
                println!("Ram[{}]", ram_addr);

                self.ram[ram_addr as usize] = self.registers[reg_addr as usize];
                self.debug_print(format_args!(
                    "\nSTO -> R{}: ({}) -> RAM[{}]",
                    reg_addr as i32, self.registers[reg_addr as usize], ram_addr as i32
                ));
            }
            15 => {
                self.halt = true;
            }
            _ => {
                return;
            }
        }
    }

    fn run(&mut self, bin: &Vec<u32>, cycle_delay_ms: u16) {
        self.load_program(bin);
        println!("====================");
        println!("Program loaded.");
        println!("====================");

        thread::sleep(Duration::new(1, 0));

        loop {
            println!("====================");
            println!("[PC -> {}]", self.program_counter);

            self.fetch_instruction();

            self.execute_instruction();

            if self.program_counter == 63 || self.halt {
                self.program_counter = 0;
                self.halt = false;
                break;
            }

            self.program_counter += 1;

            println!("====================");

            thread::sleep(Duration::from_millis(cycle_delay_ms as u64));
        }

        println!("======================");
        println!("Execution finished.");
        println!("======================");
    }

    fn demo(&mut self) {
        let program: Vec<u32> = vec![
            0b_0001_0000000000000001_01,   // 1 | lod 1 r1           load '1' into R1
            0b_0001_0000000000000001_10,   // 2 | lod 1 r2           load '1' into R2
            0b_0010_000000000000_01_10_10, // 3 | add r1, r2 -> r2   add R1 and R2 and save result to R2
            0b_0100_0000000000001000_10, // 4 | cmp 8 r2           compare value of R2 to '8' and save compare result to flag
            0b_1000_0000000000000_00010, // 5 | jlt 2              jump to line 2 if flag comparison is R2 < 8
            0b_1001_0000000000_010010_10, // 6 | sto 18 r2          store value of R2 to address 18 in RAM
            0b_1111_000000000000000000,   // 7 | hlt                halt execution
        ];

        self.run(&program, 250);
    }
}

fn assemble(program_str: &str) -> Vec<u32> {
    let program = program_str.parse::<String>().unwrap();
    // Construct machine code instruction based on input terms

    // Separate instructions into collection
    let input_instructions: Vec<&str> = program.split('\n').collect();
    let mut output_instructions: Vec<u32> = Vec::new();

    for &ins in input_instructions.iter() {
        let mut output_ins: u32 = 0b0;
        let mut opcode: u32 = 0b0;
        let mut operand: u32 = 0b0;

        // Separate terms from instruction
        let terms: Vec<&str> = ins.split_whitespace().collect::<Vec<&str>>();

        // Replace opcode term with its corresponding machine code
        match terms[0] {
            "lod" => output_ins = 0b0001,
            "add" => output_ins = 0b0010,
            "sub" => output_ins = 0b0011,
            "cmp" => output_ins = 0b0100,
            "jmp" => output_ins = 0b0101,
            "jeq" => output_ins = 0b0110,
            "jgt" => output_ins = 0b0111,
            "jlt" => output_ins = 0b1000,
            "sto" => output_ins = 0b1001,
            "hlt" => output_ins = 0b1111,
            _ => {}
        }

        opcode = output_ins;
        
        output_ins <<= 18;

        // Parse operand depending on opcode
        match (opcode) {
            // Extract immediate value and register address from address string
            1 => {
                operand = terms[1].chars().collect::<String>().parse::<u32>().unwrap() << 2;
                operand |= terms[2]
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();

                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            2 => {
                operand = terms[1]
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap()
                    << 4;
                operand |= terms[2]
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap()
                    << 2;
                operand |= terms[3]
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap();
                
                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            15 => {
                output_instructions.push(output_ins);
            }
            _ => {
                panic!("Failed to assemble input.");
            }
        }
    }

    println!("==Assembler==");
    println!("Input program: {:?}", input_instructions);
    println!("Assembled program: ");

    for (idx, ins) in output_instructions.iter().enumerate() {
        println!("[{}] - \t{:022b}", idx, ins);
    }

    output_instructions
}

fn main() {
    let mut cpu = Processor::new(true);
    cpu.demo();

    let assembled_program: Vec<u32> = assemble("lod 5 r1\nlod 4 r2\nadd r1 r2 r2\nhlt");
    
    println!("Assembled program: {:022b}", &assembled_program[0]);
    
    cpu.run(&assembled_program, 500);
    
}
