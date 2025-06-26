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

    // memory: 2^16 bytes
    memory: [u8; 0x10000],
}

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
        self.memory[(Self::PROGRAM_MEM_START)..(Self::PROGRAM_MEM_START + program.len())].fill(0);
        self.memory[(Self::PROGRAM_MEM_START)..(Self::PROGRAM_MEM_START + program.len())]
            .copy_from_slice(program);
    }

    pub fn get_state(&self) -> String {
        format!(
            "r0: {:#X}\nr1: {:#X}\nr2: {:#X}\npc: {:#X}\nsp: {:#X}",
            self.r0, self.r1, self.r2, self.pc, self.sp
        )
    }

    pub fn get_display_addresses(&self) -> (*const u8, *const u8) {
        let display_control = &self.memory[Self::DISPLAY_CTRL_ADDR] as *const u8;
        let display_data = &self.memory[Self::DISPLAY_DATA_ADDR] as *const u8;
        (display_control, display_data)
    }

    pub fn step(&mut self) {
        use crate::program::{ArithmeticOp, Instruction, Register};

        // Fetch
        let instruction = unsafe {
            let mut instruction_addr = &self.memory[self.pc as usize] as *const u8;
            let instr_1 = instruction_addr.read_unaligned() as u32;
            instruction_addr = instruction_addr.add(1);
            let instr_2 = instruction_addr.read_unaligned() as u32;
            instruction_addr = instruction_addr.add(1);
            let instr_3 = instruction_addr.read_unaligned() as u32;
            instr_1 << 16 | instr_2 << 8 | instr_3
        };
        println!(
            "Executing instruction at {:#X}: {:#b}",
            self.pc, instruction
        );
        let decoded_instruction = Instruction::from_binary(instruction);
        println!("Decoded instruction: {:?}", decoded_instruction);
        self.pc += 3; // Instruction size : 24 bits
    }

    fn memory(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    fn memory_mut(&mut self) -> *mut u8 {
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
            for _ in 0..10 {
                // Todo: not only 10 cycles
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
