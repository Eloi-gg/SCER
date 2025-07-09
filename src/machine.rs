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
    memory: [u16; 0xFFFF + 1],
    display_ctrl: u8,
    display_data: u8,
}

use std::fmt::Debug;

use crate::program::{ArithmeticOp, Instruction, Register};

/// Memory layout:
/// 0x0000 - 0x3FFF: program memory
/// 0x4000 - 0x5FFF: stack memory
/// 0x6000 - 0xBFFF: heap memory
/// 0xC000 - 0xEFFF: IO memory
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
    const TEST_MEM_START: usize = 0xF000;
    pub const MEMORY_END: usize = 0xFFFF;

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

            pc: Self::PROGRAM_MEM_START as u16,
            sp: Self::STACK_MEM_START as u16,

            memory: [0; 0x10000],
            display_ctrl: 0,
            display_data: 0,
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

    pub fn reset_registers(&mut self) {
        self.r0 = 0;
        self.r1 = 0;
        self.r2 = 0;
        self.a0 = 0;
        self.a1 = 0;
        self.a2 = 0;
        self.z = 0;
        self.f = 0;

        self.pc = Self::PROGRAM_MEM_START as u16;
        self.sp = Self::STACK_MEM_START as u16;
    }

    pub fn get_display_addresses(&self) -> (*const u8, *const u8) {
        (
            &self.display_data as *const u8,
            &self.display_ctrl as *const u8,
        )
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
        let decoded_instruction = Instruction::from_binary(instruction);

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
        // TODO: Implement all instructions and add tests
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
            Instruction::Or(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value_a = self.get_register_value(reg_a);
                    let result = value_a | imm as u16;
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a | value_b;
                    self.set_register_value(dest, result);
                }
            },
            Instruction::Xor(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value_a = self.get_register_value(reg_a);
                    let result = value_a ^ imm as u16;
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a ^ value_b;
                    self.set_register_value(dest, result);
                }
            },
            Instruction::Asl(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value = self.get_register_value(reg_a);
                    let result = value.wrapping_shl(imm as u32);
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a.wrapping_shl(value_b as u32);
                    self.set_register_value(dest, result);
                }
            },
            Instruction::Asr(op) => match op {
                ArithmeticOp::Immediate { dest, reg_a, imm } => {
                    let value = self.get_register_value(reg_a);
                    let result = value.wrapping_shr(imm as u32);
                    self.set_register_value(dest, result);
                }
                ArithmeticOp::Register { dest, reg_a, reg_b } => {
                    let value_a = self.get_register_value(reg_a);
                    let value_b = self.get_register_value(reg_b);
                    let result = value_a.wrapping_shr(value_b as u32);
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
                // TODO
                crate::program::TwoArgsOp::Immediate(reg, imm) => {
                    let address = imm;
                    let value = self.get_memory(address);
                    self.set_register_value(reg, value);
                }
                crate::program::TwoArgsOp::Register(reg, addr_reg) => {
                    let address = self.get_register_value(addr_reg);
                    let value = self.get_memory(address);
                    self.set_register_value(reg, value);
                }
            },
            Instruction::Sw(op) => match op {
                // TODO
                crate::program::TwoArgsOp::Immediate(reg, imm) => {
                    let address = imm;
                    let value = self.get_register_value(reg);
                    self.set_memory(address, value);
                }
                crate::program::TwoArgsOp::Register(reg, addr_reg) => {
                    let address = self.get_register_value(addr_reg);
                    let value = self.get_register_value(reg);
                    self.set_memory(address, value);
                }
            },
            Instruction::Jeq(op) => match op {
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
            },
            Instruction::Jlt(op) => match op {
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
            },
            Instruction::Jne(op) => match op {
                crate::program::OtherOp::Immediate(imm) => {
                    if self.f & Self::ZERO_FLAG == 0 {
                        self.pc = (imm - 3) as u16;
                    }
                }
                crate::program::OtherOp::Register(reg) => {
                    if self.f & Self::ZERO_FLAG == 0 {
                        self.pc = self.get_register_value(reg) - 3;
                    }
                }
            },
            Instruction::Push(op) => match op {
                crate::program::OtherOp::Immediate(imm) => {
                    self.set_memory(self.sp, imm);
                    self.sp += 1;
                    assert!(self.sp < Self::HEAP_MEM_START as u16, "Stack overflow");
                }
                crate::program::OtherOp::Register(reg) => {
                    let value = self.get_register_value(reg);
                    self.set_memory(self.sp, value);
                    self.sp += 1;
                    assert!(self.sp < Self::HEAP_MEM_START as u16, "Stack overflow");
                }
            }
            Instruction::Pop(reg) => {
                assert!(self.sp > Self::STACK_MEM_START as u16, "Stack underflow");
                self.sp -= 1;
                let value = self.get_memory(self.sp);
                self.set_register_value(reg, value);
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

    pub fn set_memory(&mut self, address: u16, value: u16) {
        println!(
            "Setting memory at address {:#06X} to value {:#06X}",
            address, value
        );
        assert!(address <= Self::MEMORY_END as u16, "Address out of bounds");
        if address == Self::DISPLAY_CTRL_ADDR as u16 {
            self.display_ctrl = value as u8;
        } else if address == Self::DISPLAY_DATA_ADDR as u16 {
            self.display_data = value as u8;
        } else {
            unsafe {
                *(self.memory_mut().add(address as usize) as *mut u16) = value;
            }
        }
    }

    pub fn get_memory(&self, address: u16) -> u16 {
        if address == Self::DISPLAY_CTRL_ADDR as u16 {
            return self.display_ctrl as u16;
        } else if address == Self::DISPLAY_DATA_ADDR as u16 {
            return self.display_data as u16;
        }
        assert!(address <= Self::MEMORY_END as u16, "Address out of bounds");
        unsafe { *(self.memory().add(address as usize) as *const u16) }
    }
}

impl Debug for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instruction = self.fetch();
        let decoded_instruction = Instruction::from_binary(instruction);

        write!(
            f,
            "-------------------------------\n\
            Next instruction: \n\
            {:#034b} \n\
            {:?}\n\
            -------------------------------\n\
            Registers:\n\
             R0: {:#06X}, R1: {:#06X}, R2: {:#06X}\n\
             A0: {:#06X}, A1: {:#06X}, A2: {:#06X}\n\
             Z:  {:#06X}, F:  {:#06X}\n\
             PC: {:#06X}, SP: {:#06X}\n\
            -------------------------------\n",
            instruction,
            decoded_instruction,
            self.r0,
            self.r1,
            self.r2,
            self.a0,
            self.a1,
            self.a2,
            self.z,
            self.f,
            self.pc,
            self.sp
        )
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
            for _ in 0..1000 {
                machine.step();
                if *test_end_addr != 0 {
                    return true;
                }
            }
            return false;
        }
    }

    #[test]
    fn stack_operations() {
        const PROGRAM_OUTPUT_ADDR: u16 = 0xF000;

        let instructions = "
!test_end_address 0xFFFF
mov $a0 0xF000 # Set output address
mov $r0 123
push $r0 # register to stack
push 234 # immediate value to stack
pop $r1 
sw $r1 $a0 # Store popped value to output address
add $a0 $a0 1 # Increment output address
push $a0 # Push incremented address to stack
pop $r1 
sw $r1 $a0 # Store popped value to output address
add $a0 $a0 1 # Increment output address
pop $r1 
sw $r1 $a0 # Store popped value to output address
add $a0 $a0 1 # Increment output address
# program end
mov $a2 0xFF    
sw  $a2 test_end_address
        ";

        let mut machine = Machine::new();
        let program = ScerProgram::compile(instructions.to_owned()).unwrap();
        machine.load(&program);
        wait_for_test_end(&mut machine);

        let expected_output = vec![234, 0xF001, 123];
        for (i, expected_value) in expected_output.into_iter().enumerate() {
            unsafe {
                let output_addr = machine.memory().add(PROGRAM_OUTPUT_ADDR as usize + i);
                assert_eq!(*output_addr, expected_value as u16, "Output mismatch at index {}", i);
            }
        }
    }

    #[test]
    fn fibonacci() {
        const PROGRAM_INPUT_ADDR: u16 = 0xF000; // Number of Fibonacci terms to compute
        const PROGRAM_OUTPUT_ADDR: u16 = 0xF002; // Output address for Fibonacci numbers

        let mut machine = Machine::new();
        let instructions = std::fs::read_to_string("./programs/fibonacci.sp").unwrap();
        let program = ScerProgram::compile(instructions).unwrap();
        machine.load(&program);

        machine.set_memory(PROGRAM_INPUT_ADDR, 10);
        assert!(wait_for_test_end(&mut machine));
        unsafe {
            let output_addr = machine.memory().add(PROGRAM_OUTPUT_ADDR as usize);
            let mut fib_numbers = Vec::new();
            for i in 0..10 {
                fib_numbers.push(*output_addr.add(i * 2) as u16);
            }
            assert_eq!(fib_numbers, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
        }

        machine.reset_registers();
        machine.set_memory(TEST_END_ADDRESS, 0);
        machine.set_memory(PROGRAM_INPUT_ADDR, 24); // Compute the first 24 Fibonacci numbers
        assert!(wait_for_test_end(&mut machine));
        unsafe {
            let output_addr = machine.memory().add(PROGRAM_OUTPUT_ADDR as usize);
            let mut fib_numbers = Vec::new();
            for i in 0..24 {
                fib_numbers.push(*output_addr.add(i * 2) as u16);
            }
            assert_eq!(
                fib_numbers,
                vec![
                    0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584,
                    4181, 6765, 10946, 17711, 28657
                ]
            );
        }
    }
}
