pub struct Machine {
    // registers
    r0: u16, // general purpose, arg0
    r1: u16, // general purpose, arg1
    r2: u16, // general purpose, arg2
    a0: u16, // general purpose
    a1: u16, // general purpose
    a2: u16, // general purpose
    z: u16,  // return value, link register
    f: u16,  // flags

    // special registers
    pc: u16,
    sp: u16,

    // memory: 2^16 bytes of size 16
    memory: [u16; 0x10000],
}

use crate::program::{ArithmeticOp, Instruction, Register};

/// Memory layout:
/// 0x0000 - 0x3FFF: program memory
/// 0x4000 - 0x5FFF: stack memory
/// 0x6000 - 0xBFFF: heap memory
/// 0xC000 - 0xEFFF: io memory
/// 0xF000 - 0xFFFF: testing

/// // IO memory layout:
/// // 0xC000: display control
/// // 0xC001: display data
/// // 0xD000: keyboard control

impl Machine {
    const PROGRAM_MEM_START: usize = 0x0000;
    const STACK_MEM_START: usize = 0x4000;
    const HEAP_MEM_START: usize = 0x6000;
    const IO_MEM_START: usize = 0xC000;

    const DISPLAY_CTRL_ADDR: usize = 0xC000;
    const DISPLAY_DATA_ADDR: usize = 0xC001;

    const ZERO_FLAG: u16 = 0b0001;
    const NEGATIVE_FLAG: u16 = 0b0010;

    pub fn new() -> Machine {
        Machine {
            r0: 0,
            r1: 0,
            r2: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            z: 0,
            f: 0,

            pc: 0,
            sp: 0,

            memory: [0; 0x10000],
        }
    }

    pub fn load(&mut self, program: &[u8]) {
        assert!(program.len() < Self::STACK_MEM_START);
        unsafe {
            std::ptr::copy_nonoverlapping(
                program.as_ptr(),
                self.memory_mut() as *mut u8,
                program.len(),
            );
        }
    }

    pub fn get_state(&self) -> String {
        let instruction = self.fetch();
        let decoded_instruction = Instruction::from_binary(instruction);

        format!(
            "-------------------------------\n\
            Next instruction: \n\
            {:#b} \n\
            {:?}\n\
            -------------------------------\n\
            Registers:\n\
             R0: {:#06X}, R1: {:#06X}, R2: {:#06X}\n\
             A0: {:#06X}, A1: {:#06X}, A2: {:#06X}\n\
             Z:  {:#06X}, F:  {:#06X}\n\
             PC: {:#06X}, SP: {:#06X}\n\
            -------------------------------\n",
            instruction, decoded_instruction,
            self.r0, self.r1, self.r2,
            self.a0, self.a1, self.a2,
            self.z, self.f,
            self.pc, self.sp
        )
    }

    pub fn get_display_addresses(&self) -> (*const u16, *const u16) {
        let display_control = &self.memory[Self::DISPLAY_CTRL_ADDR];
        let display_data = &self.memory[Self::DISPLAY_DATA_ADDR];
        (display_control, display_data)
    }

    fn fetch(&self) -> u32 {
        unsafe {
            let u8_mem = &self.memory as *const [u16] as *const u8;
            let mut instruction_addr = u8_mem.add(self.pc as usize) as *const u8;
            let instr_1 = instruction_addr.read_unaligned() as u32;
            instruction_addr = instruction_addr.add(1);
            let instr_2 = instruction_addr.read_unaligned() as u32;
            instruction_addr = instruction_addr.add(1);
            let instr_3 = instruction_addr.read_unaligned() as u32;
            instr_1 << 16 | instr_2 << 8 | instr_3
        }
    }

    pub fn step(&mut self) {
        use crate::program::{ArithmeticOp, Instruction, Register};

        // Fetch
        let instruction = self.fetch();
        println!(
            "Executing instruction at {:#X}: {:#b}",
            self.pc, instruction
        );
        let decoded_instruction = Instruction::from_binary(instruction);
        println!("Decoded instruction: {:?}", decoded_instruction);

        // Execute
        self.execute(decoded_instruction);

        // Update program counter
        self.pc += 3; // Instruction size : 24 bits
    }

    pub fn get_register_value(&self, reg: Register) -> u16 {
        match reg {
            Register::R0 => self.r0,
            Register::R1 => self.r1,
            Register::R2 => self.r2,
            Register::A0 => self.a0,
            Register::A1 => self.a1,
            Register::A2 => self.a2,
            Register::Z => self.z,
            Register::F => self.f,
        }
    }

    pub fn set_register_value(&mut self, reg: Register, value: u16) {
        match reg {
            Register::R0 => self.r0 = value,
            Register::R1 => self.r1 = value,
            Register::R2 => self.r2 = value,
            Register::A0 => self.a0 = value,
            Register::A1 => self.a1 = value,
            Register::A2 => self.a2 = value,
            Register::Z => self.z = value,
            Register::F => self.f = value,
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value_a = self.get_register_value(reg_a);
                    let result = value_a.wrapping_add(imm as u16);
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a.wrapping_add(value_b);
                    self.set_register_value(dest, result);
                }
            },
            Instruction::Sub(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value_a = self.get_register_value(reg_a);
                    let result = value_a.wrapping_sub(imm as u16);
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a.wrapping_sub(value_b);
                    self.set_register_value(dest, result);
                }
            },
            Instruction::And(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value_a = self.get_register_value(reg_a);
                    let result = value_a & imm as u16;
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a & value_b;
                    self.set_register_value(dest, result);
                }
            },
            Instruction::Cmp(op) => {
                match op {
                    crate::program::TwoArgsOp::Immediate(reg, imm) => {
                        let value = self.get_register_value(reg);
                        let result = value.wrapping_sub(imm as u16);
                        self.f = 0;
                        if result == 0 {
                            self.f |= Self::ZERO_FLAG; // Zero flag
                        }
                        if result & 0x8000 != 0 {
                            self.f |= Self::NEGATIVE_FLAG; // Negative flag
                        }
                    }
                    crate::program::TwoArgsOp::Register(reg_a, reg_b) => {
                        let value_a = self.get_register_value(reg_a);
                        let value_b = self.get_register_value(reg_b);
                        let result = value_a.wrapping_sub(value_b);
                        self.f = 0;
                        if result == 0 {
                            self.f |= 0b0001; // Zero flag
                        }
                        if result & 0x8000 != 0 {
                            self.f |= 0b0010; // Negative flag
                        }
                    }
                }
            }
            Instruction::Mov(reg, imm) => {
                let value = imm as u16;
                self.set_register_value(reg, value);
            }
            Instruction::Lw(op) => match op {
                crate::program::TwoArgsOp::Immediate(reg, imm) => {
                    let address = imm as usize;
                    let value = unsafe { *(self.memory.as_ptr().add(address) as *const u16) };
                    self.set_register_value(reg, value);
                }
                crate::program::TwoArgsOp::Register(reg, addr_reg) => {
                    let address = self.get_register_value(addr_reg) as usize;
                    let value = unsafe { *(self.memory.as_ptr().add(address) as *const u16) };
                    self.set_register_value(reg, value);
                }
            },
            Instruction::Sw(op) => match op {
                crate::program::TwoArgsOp::Immediate(reg, imm) => {
                    let address = imm as usize;
                    let value = self.get_register_value(reg);
                    unsafe {
                        *(self.memory.as_mut_ptr().add(address) as *mut u16) = value;
                    }
                }
                crate::program::TwoArgsOp::Register(reg, addr_reg) => {
                    let address = self.get_register_value(addr_reg) as usize;
                    let value = self.get_register_value(reg);
                    unsafe {
                        *(self.memory.as_mut_ptr().add(address) as *mut u16) = value;
                    }
                }
            },
            Instruction::Jeq(op) => {
                match op {
                    crate::program::OtherOp::Immediate(imm) => {
                        if self.f & Self::ZERO_FLAG != 0 {
                            self.pc = (imm - 3) as u16;
                        }
                    }
                    crate::program::OtherOp::Register(reg) => {
                        if self.f & Self::ZERO_FLAG != 0 {
                            self.pc = self.get_register_value(reg) - 3;
                        }
                    }
                }
            }
            Instruction::Jlt(op) => {
                match op {
                    crate::program::OtherOp::Immediate(imm) => {
                        if self.f & Self::NEGATIVE_FLAG == 0 {
                            self.pc = (imm - 3) as u16;
                        }
                    }
                    crate::program::OtherOp::Register(reg) => {
                        if self.f & Self::NEGATIVE_FLAG == 0 {
                            self.pc = self.get_register_value(reg) - 3;
                        }
                    }
                }
            }

            _ => {
                unimplemented!()
            }
        }
    }

    fn memory(&self) -> *const u16 {
        self.memory.as_ptr()
    }

    fn memory_mut(&mut self) -> *mut u16 {
        self.memory.as_mut_ptr()
    }
}

#[cfg(test)]
mod programs {
    use crate::machine::Machine;
    use crate::program::ScerProgram;

    const TEST_END_ADDRESS: u16 = 0xFFFF;

    fn wait_for_test_end(machine: &mut Machine) -> bool {
        unsafe {
            let test_end_addr = machine.memory().add(TEST_END_ADDRESS as usize);
            for _ in 0..100 {
                // Todo: not only 100 cycles
                machine.step();
                if *test_end_addr != 0 {
                    return true;
                }
            }
            return false;
        }
    }

    #[test]
    fn fibonacci() {
        const PROGRAM_INPUT_ADDR: u16 = 0xF000; // Number of Fibonacci terms to compute
        const PROGRAM_OUTPUT_ADDR: u16 = 0xF002; // Output address for Fibonacci numbers

        let mut machine = Machine::new();
        let instructions = std::fs::read_to_string("./programs/fibonacci.sp").unwrap();
        let program = ScerProgram::compile(instructions).unwrap();
        println!("Program: {:?}", program);
        // 0b1100_0___100_1111000000000010
        machine.load(&program);
        // unsafe  {
        //     let input_addr = machine.memory_mut().add(PROGRAM_INPUT_ADDR as usize);
        //     input_addr.write(10u8); // Input: Compute the first 10 Fibonacci numbers
        // }

        assert!(wait_for_test_end(&mut machine));

        unsafe {
            let output_addr = machine.memory().add(PROGRAM_OUTPUT_ADDR as usize);
            let mut fib_numbers = Vec::new();
            for i in 0..10 {
                fib_numbers.push(*output_addr.add(i * 2) as u16);
            }
            assert_eq!(fib_numbers, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
        }
    }
}
