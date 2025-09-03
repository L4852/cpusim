use std::fmt::Arguments;
use std::fs::File;
use std::io::Write;
use std::ops::Add;
use std::thread;
use std::time::Duration;

struct Processor {
    program_counter: usize,
    ram: [u32; 64],
    flag_register: u32,
    registers: [u32; 4],
    halt: bool,
    debug: bool,
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
        if self.debug == true {
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
                    self.debug_print(format_args!(
                        "\n{} -> [{}] (GT)",
                        get_opcode_name(4, true),
                        self.flag_register
                    ));
                } else if result == 0 {
                    self.flag_register = 2;
                    self.debug_print(format_args!(
                        "\n{} -> [{}] (EQ)",
                        get_opcode_name(4, true),
                        self.flag_register
                    ));
                } else if result < 0 {
                    self.flag_register = 3;
                    self.debug_print(format_args!(
                        "\n{} -> [{}] (LT)",
                        get_opcode_name(4, true),
                        self.flag_register
                    ));
                } else {
                    self.flag_register = 0;
                }
            }
            // Jump
            5 => {
                let jump_addr = operand & (0b11111);

                self.program_counter = jump_addr as usize;

                self.debug_print(format_args!(
                    "\n{} -> [{}]",
                    get_opcode_name(5, true),
                    self.program_counter
                ));
            }
            // Jump if equal
            6 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 2 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!(
                    "\n{} -> [{}]",
                    get_opcode_name(6, true),
                    self.program_counter
                ));
            }
            7 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 1 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!(
                    "\n{} -> [{}]",
                    get_opcode_name(7, true),
                    self.program_counter
                ));
            }
            8 => {
                let jump_addr = operand & (0b11111);

                if self.flag_register == 3 {
                    self.program_counter = jump_addr as usize - 1;
                    self.flag_register = 0;
                }
                self.debug_print(format_args!(
                    "\n{} -> [{}]",
                    get_opcode_name(8, true),
                    self.program_counter
                ));
            }
            // Store from register to RAM
            9 => {
                let reg_addr = operand & (0b11);
                let ram_addr = (operand & (0b111111 << 2)) >> 2;
                println!("Ram[{}]", ram_addr);

                self.ram[ram_addr as usize] = self.registers[reg_addr as usize];
                self.debug_print(format_args!(
                    "\n{} -> | R{}: ({}) -> RAM[{}]",
                    get_opcode_name(9, true),
                    reg_addr as i32,
                    self.registers[reg_addr as usize],
                    ram_addr as i32
                ));
            }
            10 => {
                let reg_addr = operand & (0b11);
                let ram_addr = (operand & (0b111111 << 2)) >> 2;
                println!("R[{}]", reg_addr);

                self.registers[reg_addr as usize] = self.ram[ram_addr as usize];
                self.debug_print(format_args!(
                    "\n{} -> | RAM[{}] ({}) -> R{}",
                    get_opcode_name(9, true),
                    ram_addr as i32,
                    self.registers[reg_addr as usize],
                    reg_addr as i32
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

    pub(crate) fn run(&mut self, bin: &Vec<u32>, cycle_delay_ms: u16) {
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
        let opcode: u32;
        let mut operand: u32;

        // Separate terms from instruction
        let terms: Vec<&str> = ins.split_whitespace().collect::<Vec<&str>>();

        // Replace opcode term with its corresponding machine code
        match terms[0] {
            "ldi" => output_ins = 0b0001,
            "add" => output_ins = 0b0010,
            "sub" => output_ins = 0b0011,
            "cmp" => output_ins = 0b0100,
            "jmp" => output_ins = 0b0101,
            "jeq" => output_ins = 0b0110,
            "jgt" => output_ins = 0b0111,
            "jlt" => output_ins = 0b1000,
            "sto" => output_ins = 0b1001,
            "lod" => output_ins = 0b1010,
            "hlt" => output_ins = 0b1111,
            _ => {}
        }

        opcode = output_ins;

        output_ins <<= 18;

        // Parse operand depending on opcode
        match opcode {
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
            3 => {
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
            4 => {
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
            5 => {
                operand = terms[1].chars().collect::<String>().parse::<u32>().unwrap();
                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            6 => {
                operand = terms[1].chars().collect::<String>().parse::<u32>().unwrap();
                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            7 => {
                operand = terms[1].chars().collect::<String>().parse::<u32>().unwrap();
                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            8 => {
                operand = terms[1].chars().collect::<String>().parse::<u32>().unwrap();
                output_ins = output_ins | operand;
                output_instructions.push(output_ins);
            }
            9 => {
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
            10 => {
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

fn get_opcode_name(opcode: u32, use_uppercase: bool) -> String {
    let mut name: String = String::from(match opcode {
        0 => "nop",
        1 => "ldi",
        2 => "add",
        3 => "sub",
        4 => "cmp",
        5 => "jmp",
        6 => "jgt",
        7 => "jeq",
        8 => "jlt",
        9 => "sto",
        10 => "lod",
        15 => "hlt",
        _ => "",
    });

    if use_uppercase {
        name = name.to_ascii_uppercase();
    }

    name
}

fn get_opcode_name_long(opcode: u32) -> String {
    let name: String = String::from(match opcode {
        0 => "NO-OP",
        1 => "LOAD IMMEDIATE",
        2 => "ADD",
        3 => "SUBTRACT",
        4 => "COMPARE",
        5 => "JUMP",
        6 => "JUMP IF GREATER THAN",
        7 => "JUMP IF EQUAL",
        8 => "JUMP IF LESS THAN",
        9 => "STORE",
        10 => "LOAD",
        15 => "HALT",
        _ => "",
    });

    let output = name.add("\n");
    output
}

pub(crate) fn print_as_assembly(instruction: u32) {
    let opcode = instruction >> 18;

    // Take lower 18 bits
    let operand = instruction & (!(0b1111 << 18));

    let mut final_string = String::new();

    final_string.push_str(get_opcode_name_long(opcode).as_str());

    match opcode {
        0 => {}
        1 => {
            let immediate_value = operand >> 2;
            let target_register = operand & (!(0b1111 << 2));

            final_string.push_str(&u32::to_string(&immediate_value));
            final_string.push_str(" R");
            final_string.push_str(&u32::to_string(&target_register));
        }
        2 => {}
        3 => {}
        4 => {}
        5 => {}
        6 => {}
        7 => {}
        8 => {}
        9 => {}
        15 => {}
        _ => {}
    }

    println!("{}", final_string);
}

fn print_program_as_assembly(program: &Vec<u32>) {
    for &ins in program.iter() {
        print_as_assembly(ins);
    }
}

fn print_raw_bytes_as_hex(bytes: &Vec<u8>) {
    for byte in bytes {
        print!("{:02X} ", byte);
    }
}

fn print_machine_code_bin(machine_code: &Vec<u32>) {
    for (idx, ins) in machine_code.iter().enumerate() {
        println!("({}) - [{:022b}]", idx, ins);
    }
}

// Convert machine code to raw bytes in Little-Endian
fn machine_code_as_bin_raw(program: &Vec<u32>) -> Vec<u8> {
    let raw_bytes: Vec<u8> = program.iter().flat_map(|x| x.to_le_bytes()).collect();
    raw_bytes
}

// Convert raw bytes to machine code
fn bin_raw_as_machine_code(bytes: &Vec<u8>) -> Vec<u32> {
    let machine_code: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
        .collect();
    machine_code
}

fn write_bytes_to_file(data: Vec<u8>) {
    let mut file = File::create("test.bin").unwrap();
    file.write_all(&data).unwrap();
}

fn assemble_from_file() -> Vec<u32> {
    let file = std::fs::read_to_string("src/test_files/test.asm").unwrap();
    let assembled = assemble(&file);

    assembled
}

fn main() {
    let mut cpu = Processor::new(true);
    // cpu.demo();
    //

    let program: Vec<u32> = assemble_from_file();

    cpu.run(&program, 0);
}
