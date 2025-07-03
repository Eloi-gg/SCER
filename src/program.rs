use std::collections::HashMap;

pub(crate) struct ScerProgram {
    instructions: Vec<Instruction>,
}

/// Instruction format (24 bits):
/// Arithmetic operations:
///     [op - 4 bits][type = 1][padding - 5 bits][reg_d - 3 bits][reg_a - 3 bits][imm - 8 bits]
///     [op - 4 bits][type = 0][padding - 10 bits][reg_d - 3 bits][reg_a - 3 bits][reg_b - 3 bits]
/// Comparison operations:
///    [op - 4 bits][type = 1][reg_a - 3 bits][imm - 16 bits]
///    [op - 4 bits][type = 0][padding - 13 bits][reg_a - 3 bits][reg_b - 3 bits]
/// Stack/Jump operations
///     [op - 4 bits][type = 1][padding - 3 bits][imm - 16 bits]
///     [op - 4 bits][type = 0][padding - 16 bits][reg_d - 3 bits]
///

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Register {
    R0,
    R1,
    R2,
    A0,
    A1,
    A2,
    Z,
    F,
}

impl Register {
    fn try_from(s: &str) -> Result<Register, ParsingError> {
        if !s.starts_with('$') {
            return Err(ParsingError::IsNotRegister(s.to_owned()));
        }
        match s {
            "$r0" => Ok(Register::R0),
            "$r1" => Ok(Register::R1),
            "$r2" => Ok(Register::R2),
            "$a0" => Ok(Register::A0),
            "$a1" => Ok(Register::A1),
            "$a2" => Ok(Register::A2),
            "$z" => Ok(Register::Z),
            "$f" => Ok(Register::F),
            _ => Err(ParsingError::InvalidRegister),
        }
    }
}

fn try_immediate(s: &str) -> Result<u16, ParsingError> {
    if s.starts_with("0x") {
        u16::from_str_radix(&s[2..], 16).map_err(|_| ParsingError::InvalidInstruction)
    } else {
        s.parse::<u16>()
            .map_err(|_| ParsingError::InvalidInstruction)
    }
}

#[derive(PartialEq)]
pub enum ArithmeticOp {
    Immediate {
        dest: Register,
        reg_a: Register,
        imm: u8,
    },
    Register {
        dest: Register,
        reg_a: Register,
        reg_b: Register,
    },
}

impl ArithmeticOp {
    fn try_new(parts: &Vec<&str>) -> Result<ArithmeticOp, ParsingError> {
        if parts.len() != 3 {
            return Err(ParsingError::InvalidNumberOfArguments);
        }
        let dest = Register::try_from(parts[0])?; // Dest must be a register
        let reg_a = Register::try_from(parts[1])?; // Reg A must be a register

        return match Register::try_from(parts[2]) {
            Ok(reg_b) => Ok(ArithmeticOp::Register { dest, reg_a, reg_b }),
            Err(e) => match e {
                ParsingError::IsNotRegister(_) => match try_immediate(parts[2]) {
                    Ok(imm) => Ok(ArithmeticOp::Immediate {
                        dest,
                        reg_a,
                        imm: imm as u8,
                    }),
                    Err(_) => Err(ParsingError::InvalidInstruction),
                },
                _ => Err(e),
            },
        };
    }

    fn to_binary(&self) -> u32 {
        match self {
            ArithmeticOp::Immediate { dest, reg_a, imm } => {
                return 1u32 << 19 | // type = 1
                    (*dest as u32) << 11 |
                    (*reg_a as u32) << 8 |
                    (*imm as u32);
            }
            ArithmeticOp::Register { dest, reg_a, reg_b } => {
                return 0u32 << 19 | // type = 0
                    (*dest as u32) << 6 |
                    (*reg_a as u32) << 3 |
                    (*reg_b as u32);
            }
        }
    }

    fn from_binary(binary: u32) -> Self {
        let type_bit = (binary >> 19) & 0x1;

        if type_bit == 1 {
            // Immediate
            let dest = ((binary >> 11) & 0x7) as u8;
            let reg_a = ((binary >> 8) & 0x7) as u8;
            let imm = (binary & 0xFF) as u8;
            return ArithmeticOp::Immediate {
                dest: unsafe { std::mem::transmute(dest) },
                reg_a: unsafe { std::mem::transmute(reg_a) },
                imm,
            };
        } else {
            // Register
            let dest = ((binary >> 6) & 0x7) as u8;
            let reg_a = ((binary >> 3) & 0x7) as u8;
            let reg_b = (binary & 0x7) as u8;
            return ArithmeticOp::Register {
                dest: unsafe { std::mem::transmute(dest) },
                reg_a: unsafe { std::mem::transmute(reg_a) },
                reg_b: unsafe { std::mem::transmute(reg_b) },
            };
        }
    }
}

impl std::fmt::Debug for ArithmeticOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate { dest, reg_a, imm } => write!(f, "{:?}, {:?}, {:#X}", dest, reg_a, imm),
            Self::Register { dest, reg_a, reg_b } => {
                write!(f, "{:?}, {:?}, {:?}", dest, reg_a, reg_b)
            }
        }
    }
}

#[derive(PartialEq)]
pub enum TwoArgsOp {
    Immediate(Register, u16),
    Register(Register, Register),
}

impl TwoArgsOp {
    fn try_new(parts: &Vec<&str>) -> Result<TwoArgsOp, ParsingError> {
        if parts.len() != 2 {
            return Err(ParsingError::InvalidNumberOfArguments);
        }
        let reg_a = Register::try_from(parts[0])?; // Reg A must be a register

        return match Register::try_from(parts[1]) {
            Ok(reg_b) => Ok(TwoArgsOp::Register(reg_a, reg_b)),
            Err(e) => match e {
                ParsingError::IsNotRegister(_) => match try_immediate(parts[1]) {
                    Ok(imm) => Ok(TwoArgsOp::Immediate(reg_a, imm)),
                    Err(_) => Err(ParsingError::InvalidInstruction),
                },
                _ => Err(e),
            },
        };
    }

    fn to_binary(&self) -> u32 {
        match self {
            TwoArgsOp::Immediate(reg_a, imm) => {
                return 1u32 << 19 | // type = 1
                    (*reg_a as u32) << 16 |
                    (*imm as u32);
            }
            TwoArgsOp::Register(reg_a, reg_b) => {
                return 0u32 << 19 | // type = 0
                    (*reg_a as u32) << 3 |
                    (*reg_b as u32) << 0;
            }
        }
    }

    fn from_binary(binary: u32) -> Self {
        let type_bit = (binary >> 19) & 0x1;

        if type_bit == 1 {
            // Immediate
            let reg_a = ((binary >> 16) & 0x7) as u8;
            let imm = (binary & 0xFFFF) as u16;
            return TwoArgsOp::Immediate(unsafe { std::mem::transmute(reg_a) }, imm);
        } else {
            // Register
            let reg_a = ((binary >> 3) & 0x7) as u8;
            let reg_b = (binary & 0x7) as u8;
            return TwoArgsOp::Register(unsafe { std::mem::transmute(reg_a) }, unsafe {
                std::mem::transmute(reg_b)
            });
        }
    }
}

impl std::fmt::Debug for TwoArgsOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(reg_a, imm) => write!(f, "{:?}, {:#X}", reg_a, imm),
            Self::Register(reg_a, reg_b) => write!(f, "{:?}, {:?}", reg_a, reg_b),
        }
    }
}

#[derive(PartialEq)]
pub enum OtherOp {
    Immediate(u16),
    Register(Register),
}

impl OtherOp {
    fn to_binary(&self) -> u32 {
        match self {
            OtherOp::Immediate(imm) => {
                return 1u32 << 19 | // type = 1
                    (*imm as u32);
            }
            OtherOp::Register(reg) => {
                return 0u32 << 19 | // type = 0
                    (*reg as u32);
            }
        }
    }

    fn from_binary(binary: u32) -> Self {
        let type_bit = (binary >> 19) & 0x1;

        if type_bit == 1 {
            // Immediate
            let imm = (binary & 0xFFFF) as u16;
            return OtherOp::Immediate(imm);
        } else {
            // Register
            let reg = (binary & 0x7) as u8;
            return OtherOp::Register(unsafe { std::mem::transmute(reg) });
        }
    }
}

impl std::fmt::Debug for OtherOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(imm) => write!(f, "{:#X}", imm),
            Self::Register(reg) => write!(f, "{:?}", reg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParsingError {
    EmptyLine,
    IsNotRegister(String),
    InvalidInstruction,
    InvalidNumberOfArguments,
    InvalidRegister,
    LabelDuplicate(String),
    UnknownInstruction(String),
}

#[derive(PartialEq)]
pub enum Instruction {
    /// Arithmetic operations
    Add(ArithmeticOp),
    Sub(ArithmeticOp),
    And(ArithmeticOp),
    Or(ArithmeticOp),
    Xor(ArithmeticOp),
    Asl(ArithmeticOp),
    Asr(ArithmeticOp),

    /// Comparison operations
    Cmp(TwoArgsOp),

    /// Stack operations
    Push(OtherOp),
    Pop(Register),

    /// Memory operations
    Lw(TwoArgsOp),
    Sw(TwoArgsOp),
    Mov(Register, u16),

    /// Jump operations
    Jeq(OtherOp),
    Jlt(OtherOp),
    Jne(OtherOp),
}

impl Instruction {
    fn parse(line: &str) -> Result<Instruction, ParsingError> {
        // Remove comments
        let line = line.split('#').next().unwrap_or("");
        if line.is_empty() {
            return Err(ParsingError::EmptyLine);
        }
        let mut parts = line.split_whitespace();
        let op = parts.next().unwrap();
        let mut args = parts.collect::<Vec<_>>();

        // Arithmetic operations
        if matches!(op, "add" | "sub" | "and" | "or" | "xor" | "asl" | "asr") {
            let arithmetic_op = ArithmeticOp::try_new(&args)?;
            return match op {
                "add" => Ok(Instruction::Add(arithmetic_op)),
                "sub" => Ok(Instruction::Sub(arithmetic_op)),
                "and" => Ok(Instruction::And(arithmetic_op)),
                "or" => Ok(Instruction::Or(arithmetic_op)),
                "xor" => Ok(Instruction::Xor(arithmetic_op)),
                "asl" => Ok(Instruction::Asl(arithmetic_op)),
                "asr" => Ok(Instruction::Asr(arithmetic_op)),
                _ => unreachable!(),
            };
        }

        // Comparison operations
        if "cmp" == op {
            let comp_op = TwoArgsOp::try_new(&args)?;
            return Ok(Instruction::Cmp(comp_op));
        }

        // Push and jump operations
        if matches!(op, "push" | "jeq" | "jlt" | "jne") {
            if args.len() != 1 {
                return Err(ParsingError::InvalidNumberOfArguments);
            }
            let op_content = match Register::try_from(args[0]) {
                Ok(reg) => Ok(OtherOp::Register(reg)),
                Err(e) => match e {
                    ParsingError::IsNotRegister(_) => {
                        if let Ok(imm) = try_immediate(args[0]) {
                            Ok(OtherOp::Immediate(imm))
                        } else {
                            Err(e)
                        }
                    }
                    _ => Err(e),
                },
            }?;

            return match op {
                "push" => Ok(Instruction::Push(op_content)),
                "jeq" => Ok(Instruction::Jeq(op_content)),
                "jlt" => Ok(Instruction::Jlt(op_content)),
                "jne" => Ok(Instruction::Jne(op_content)),
                _ => unreachable!(),
            };
        }

        if "pop" == op {
            if args.len() != 1 {
                return Err(ParsingError::InvalidNumberOfArguments);
            }
            return match Register::try_from(args[0]) {
                Ok(reg) => Ok(Instruction::Pop(reg)),
                Err(e) => Err(e),
            };
        }

        // Memory operations
        if matches!(op, "lw" | "sw") {
            if args.len() != 2 {
                return Err(ParsingError::InvalidNumberOfArguments);
            }
            let reg = Register::try_from(args[0])?; // Arg 0 must be a register
            let op_content = match Register::try_from(args[1]) {
                Ok(reg2) => Ok(TwoArgsOp::Register(reg, reg2)),
                Err(e) => match e {
                    ParsingError::IsNotRegister(_) => match try_immediate(args[1]) {
                        Ok(imm) => Ok(TwoArgsOp::Immediate(reg, imm)),
                        Err(_) => Err(e),
                    },
                    _ => Err(e),
                },
            };
            return match op {
                "lw" => Ok(Instruction::Lw(op_content?)),
                "sw" => Ok(Instruction::Sw(op_content?)),
                _ => unreachable!(),
            };
        }

        // Move operation
        if "mov" == op {
            if args.len() != 2 {
                return Err(ParsingError::InvalidNumberOfArguments);
            }
            let reg = Register::try_from(args[0])?;
            let imm = try_immediate(args[1])?;
            return Ok(Instruction::Mov(reg, imm));
        }

        // Unknown instruction
        Err(ParsingError::UnknownInstruction(op.to_owned()))
    }

    fn to_binary(&self, buffer: &mut [u8]) {
        use Instruction::*;

        let instruction: u32 = match self {
            Add(op) => 0x0 << 20 | op.to_binary(),
            Sub(op) => 0x1 << 20 | op.to_binary(),
            And(op) => 0x2 << 20 | op.to_binary(),
            Or(op) => 0x3 << 20 | op.to_binary(),
            Xor(op) => 0x4 << 20 | op.to_binary(),
            Asl(op) => 0x5 << 20 | op.to_binary(),
            Asr(op) => 0x6 << 20 | op.to_binary(),
            Cmp(op) => 0x7 << 20 | op.to_binary(),
            Push(op) => 0x8 << 20 | op.to_binary(),
            Pop(reg) => 0x9 << 20 | (*reg as u32),
            Lw(op) => 0xA << 20 | op.to_binary(),
            Sw(op) => 0xB << 20 | op.to_binary(),
            Mov(reg, imm) => 0xC << 20 | (*reg as u32) << 16 | *imm as u32,
            Jeq(op) => 0xD << 20 | op.to_binary(),
            Jlt(op) => 0xE << 20 | op.to_binary(),
            Jne(op) => 0xF << 20 | op.to_binary(),
        };

        buffer[0] = (instruction >> 16) as u8;
        buffer[1] = (instruction >> 8) as u8;
        buffer[2] = instruction as u8;
    }

    pub fn from_binary(binary: u32) -> Self {
        use Instruction::*;

        let op = (binary >> 20) & 0xF;
        let instruction = binary & 0xFFFFF; // 20 remaining bits

        match op {
            0x0 => Add(ArithmeticOp::from_binary(instruction)),
            0x1 => Sub(ArithmeticOp::from_binary(instruction)),
            0x2 => And(ArithmeticOp::from_binary(instruction)),
            0x3 => Or(ArithmeticOp::from_binary(instruction)),
            0x4 => Xor(ArithmeticOp::from_binary(instruction)),
            0x5 => Asl(ArithmeticOp::from_binary(instruction)),
            0x6 => Asr(ArithmeticOp::from_binary(instruction)),
            0x7 => Cmp(TwoArgsOp::from_binary(instruction)),
            0x8 => Push(OtherOp::from_binary(instruction)),
            0x9 => Pop(unsafe { std::mem::transmute((instruction & 0x7) as u8) }),
            0xA => Lw(TwoArgsOp::from_binary(instruction)),
            0xB => Sw(TwoArgsOp::from_binary(instruction)),
            0xC => Mov(
                unsafe { std::mem::transmute((instruction >> 16) as u8) },
                (instruction & 0xFFFF) as u16,
            ),
            0xD => Jeq(OtherOp::from_binary(instruction)),
            0xE => Jlt(OtherOp::from_binary(instruction)),
            0xF => Jne(OtherOp::from_binary(instruction)),
            _ => panic!("Unknown instruction: {:#X}", op),
        }
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add(arg0) => write!(f, "add {:?}", arg0),
            Self::Sub(arg0) => write!(f, "sub {:?}", arg0),
            Self::And(arg0) => write!(f, "and {:?}", arg0),
            Self::Or(arg0) =>  write!(f, "or {:?}", arg0),
            Self::Xor(arg0) => write!(f, "xor {:?}", arg0),
            Self::Asl(arg0) => write!(f, "asl {:?}", arg0),
            Self::Asr(arg0) => write!(f, "asr {:?}", arg0),
            Self::Cmp(arg0) => write!(f, "cmp {:?}", arg0),
            Self::Push(arg0) => write!(f, "push {:?}", arg0),
            Self::Pop(arg0) => write!(f, "pop {:?}", arg0),
            Self::Lw(arg0) => write!(f, "lw {:?}", arg0),
            Self::Sw(arg0) => write!(f, "sw {:?}", arg0),
            Self::Mov(arg0, arg1) => 
                write!(f, "mov {:?}, {:#X}", arg0, arg1),
            Self::Jeq(arg0) => write!(f, "jeq {:?}", arg0),
            Self::Jlt(arg0) => write!(f, "jlt {:?}", arg0),
            Self::Jne(arg0) => write!(f, "jne {:?}", arg0),
        }
    }
}
impl ScerProgram {
    fn preprocessor(code: &mut String) -> Result<(), ParsingError> {
        let mut ranges = Vec::new();

        // Replace every define with its value
        let mut defines = HashMap::new();
        for line in code.lines() {
            if line.starts_with('!') {
                let parts: Vec<&str> = line[1..].split_whitespace().collect();
                if parts.len() != 2 {
                    return Err(ParsingError::InvalidInstruction);
                }
                let key = parts[0].to_owned();
                let value = parts[1].to_owned();
                defines.insert(key, value);

                // Remove the line from the code
                let line_start = code.find(line).unwrap();
                let line_end = line_start + line.len();
                ranges.push((line_start, line_end));
            }
        }

        for range in ranges.iter().rev() {
            code.replace_range(range.0..range.1, "");
        }
        ranges.clear();

        // Replace defines with their values
        for (key, value) in defines {
            *code = code.replace(&key, &value);
        }

        *code = code
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        for range in ranges.iter().rev() {
            code.replace_range(range.0..range.1, "");
        }
        ranges.clear();

        let mut label_addresses = HashMap::new();
        let mut instruction_number: u16 = 0;
        // Labels
        for (line_num, line) in code.lines().enumerate() {
            if line.starts_with('@') {
                let label = line[1..]
                    .split_ascii_whitespace()
                    .next()
                    .unwrap()
                    .to_owned();
                if label_addresses.contains_key(&label) {
                    return Err(ParsingError::LabelDuplicate(label));
                }
                label_addresses.insert(label, instruction_number);
                let line_start = code.find(line).unwrap();
                let line_end = line_start + line.len();
                ranges.push((line_start, line_end));
            } else {
                instruction_number += 1;
            }
        }
        for range in ranges.iter().rev() {
            code.replace_range(range.0..range.1, "");
        }

        // Replace labels with their addresses
        for (label, line) in label_addresses {
            *code = code.replace(&label, &line.to_string());
        }

        Ok(())
    }

    pub fn compile(mut code: String) -> Result<Vec<u8>, ParsingError> {
        let mut out = Vec::new();
        Self::preprocessor(&mut code)?;
        let mut buffer = [0u8; 3]; // 24bits per instruction
        for line in code.lines() {
            match Instruction::parse(line) {
                Ok(instruction) => instruction.to_binary(&mut buffer),
                Err(e) => match e {
                    ParsingError::EmptyLine => continue,
                    _ => panic!("Failed to parse instruction: {:?} | {}", e, line),
                },
            }
            out.extend_from_slice(&buffer);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod compilation {
    use super::*;

    fn assert_sequence<T>(sequence_to_test: &Vec<T>, expected_sequence: &Vec<T>)
    where
        T: PartialEq + std::fmt::Debug,
    {
        assert_eq!(sequence_to_test.len(), expected_sequence.len());
        for (i, item) in sequence_to_test.iter().enumerate() {
            assert_eq!(item, &expected_sequence[i], "at index {}", i);
        }
    }

    fn parse_instructions(code: &str) -> Vec<Instruction> {
        let mut code = code.to_owned();
        let mut parsed_instructions = Vec::new();
        compilation::ScerProgram::preprocessor(&mut code).unwrap();
        for line in code.lines() {
            //
            match Instruction::parse(line) {
                Ok(instruction) => parsed_instructions.push(instruction),
                Err(e) => match e {
                    ParsingError::EmptyLine => continue,
                    _ => panic!("Failed to parse instruction: {:?} | {}", e, line),
                },
            }
        }
        parsed_instructions
    }

    #[test]
    fn parsing_arithmetic_immediate() {
        let code = "add $a0 $r1 1
sub $a1 $r0 255 # comment
and $a2 $a2 0xFF # comment b
or $r0 $f 123
xor $r1 $a2 0xFF
asl $r2 $a0 1
asr $z $z 4
# comment c
";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Add(ArithmeticOp::Immediate {
            dest: Register::A0,
            reg_a: Register::R1,
            imm: 1,
        }));
        expected_instructions.push(Instruction::Sub(ArithmeticOp::Immediate {
            dest: Register::A1,
            reg_a: Register::R0,
            imm: 255,
        }));
        expected_instructions.push(Instruction::And(ArithmeticOp::Immediate {
            dest: Register::A2,
            reg_a: Register::A2,
            imm: 0xFF,
        }));
        expected_instructions.push(Instruction::Or(ArithmeticOp::Immediate {
            dest: Register::R0,
            reg_a: Register::F,
            imm: 123,
        }));
        expected_instructions.push(Instruction::Xor(ArithmeticOp::Immediate {
            dest: Register::R1,
            reg_a: Register::A2,
            imm: 0xFF,
        }));
        expected_instructions.push(Instruction::Asl(ArithmeticOp::Immediate {
            dest: Register::R2,
            reg_a: Register::A0,
            imm: 1,
        }));
        expected_instructions.push(Instruction::Asr(ArithmeticOp::Immediate {
            dest: Register::Z,
            reg_a: Register::Z,
            imm: 4,
        }));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_arithmetic_register() {
        let code = "add $a0 $r1 $r2
sub $a1 $r0 $r2
and $a2 $a2 $r0
or $r0 $f $r2
xor $r1 $a2 $r0
asl $r2 $a0 $r1
asr $z $z $r2";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Add(ArithmeticOp::Register {
            dest: Register::A0,
            reg_a: Register::R1,
            reg_b: Register::R2,
        }));
        expected_instructions.push(Instruction::Sub(ArithmeticOp::Register {
            dest: Register::A1,
            reg_a: Register::R0,
            reg_b: Register::R2,
        }));
        expected_instructions.push(Instruction::And(ArithmeticOp::Register {
            dest: Register::A2,
            reg_a: Register::A2,
            reg_b: Register::R0,
        }));
        expected_instructions.push(Instruction::Or(ArithmeticOp::Register {
            dest: Register::R0,
            reg_a: Register::F,
            reg_b: Register::R2,
        }));
        expected_instructions.push(Instruction::Xor(ArithmeticOp::Register {
            dest: Register::R1,
            reg_a: Register::A2,
            reg_b: Register::R0,
        }));
        expected_instructions.push(Instruction::Asl(ArithmeticOp::Register {
            dest: Register::R2,
            reg_a: Register::A0,
            reg_b: Register::R1,
        }));
        expected_instructions.push(Instruction::Asr(ArithmeticOp::Register {
            dest: Register::Z,
            reg_a: Register::Z,
            reg_b: Register::R2,
        }));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_cmp() {
        let code = "# comment

        cmp $r1 1 # immediate
cmp $r0 255 # comment
cmp $a2 0xFF # comment b
cmp $r1 $r2 # register register
cmp $a0 $a0 # same register";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Cmp(TwoArgsOp::Immediate(Register::R1, 1)));
        expected_instructions.push(Instruction::Cmp(TwoArgsOp::Immediate(Register::R0, 255)));
        expected_instructions.push(Instruction::Cmp(TwoArgsOp::Immediate(Register::A2, 0xFF)));
        expected_instructions.push(Instruction::Cmp(TwoArgsOp::Register(
            Register::R1,
            Register::R2,
        )));
        expected_instructions.push(Instruction::Cmp(TwoArgsOp::Register(
            Register::A0,
            Register::A0,
        )));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_stack() {
        let code = "push $r1
push 255 # immediate
push 0xFF # comment b
pop $r2 # register";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Push(OtherOp::Register(Register::R1)));
        expected_instructions.push(Instruction::Push(OtherOp::Immediate(255)));
        expected_instructions.push(Instruction::Push(OtherOp::Immediate(0xFF)));
        expected_instructions.push(Instruction::Pop(Register::R2));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_memory() {
        let code = "lw $r1 1
lw $z 0x1234 # immediate
lw $a2 $r2 # register
sw $r0 0x0255 # immediate
sw $a0 $a0 # same register
";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Lw(TwoArgsOp::Immediate(Register::R1, 1)));
        expected_instructions.push(Instruction::Lw(TwoArgsOp::Immediate(Register::Z, 0x1234)));
        expected_instructions.push(Instruction::Lw(TwoArgsOp::Register(
            Register::A2,
            Register::R2,
        )));
        expected_instructions.push(Instruction::Sw(TwoArgsOp::Immediate(Register::R0, 0x0255)));
        expected_instructions.push(Instruction::Sw(TwoArgsOp::Register(
            Register::A0,
            Register::A0,
        )));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_move() {
        let code = "mov $r1 1
mov $z 0x1234 # immediate";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Mov(Register::R1, 1));
        expected_instructions.push(Instruction::Mov(Register::Z, 0x1234));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn parsing_jump() {
        let code = "jeq 1
jeq 0x1234 # immediate
jeq $r2 # register
jlt 0x0255 # immediate
jlt $a0 # same register
jne 0xFFFF # immediate
jne $r1 # register
";
        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Jeq(OtherOp::Immediate(1)));
        expected_instructions.push(Instruction::Jeq(OtherOp::Immediate(0x1234)));
        expected_instructions.push(Instruction::Jeq(OtherOp::Register(Register::R2)));
        expected_instructions.push(Instruction::Jlt(OtherOp::Immediate(0x0255)));
        expected_instructions.push(Instruction::Jlt(OtherOp::Register(Register::A0)));
        expected_instructions.push(Instruction::Jne(OtherOp::Immediate(0xFFFF)));
        expected_instructions.push(Instruction::Jne(OtherOp::Register(Register::R1)));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn defines() {
        let code = "
!input_address 0xF0
!output_address 0xF002
!test_end_address 0xFFFF
!out_register $r0

add $r0 $r1 input_address
sub out_register out_register 255
        ";

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Add(ArithmeticOp::Immediate {
            dest: Register::R0,
            reg_a: Register::R1,
            imm: 0xF0,
        }));
        expected_instructions.push(Instruction::Sub(ArithmeticOp::Immediate {
            dest: Register::R0,
            reg_a: Register::R0,
            imm: 255,
        }));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }

    #[test]
    fn labels() {
        let code = "
@start
add $r0 $r1 1 # addr 0

@loop

add $r0 $r1 2 # addr 1

sub $r0 $r1 3 # addr 2
@end

# @start => addr 0
# @loop => addr 1
# @end => addr 3
add $r0 $r1 2 # addr 3

jeq loop
jlt start
jeq end
";
        let mut prepprocessed_code = code.to_owned();
        compilation::ScerProgram::preprocessor(&mut prepprocessed_code).unwrap();
        println!("{}", prepprocessed_code);

        let jump_loop = Instruction::Jeq(OtherOp::Immediate(1));
        let jump_start = Instruction::Jlt(OtherOp::Immediate(0));
        let jump_end = Instruction::Jeq(OtherOp::Immediate(3));

        let parsed_instructions = parse_instructions(&code);

        assert_eq!(parsed_instructions[4], jump_loop); // found 3
        assert_eq!(parsed_instructions[5], jump_start); // found 1
        assert_eq!(parsed_instructions[6], jump_end); // found 7
    }

    #[test]
    fn binary_conversion() {
        let code = "add $r0 $r1 1
sub $a1 $a2 $r1
and $a2 $a2 $z
asr $a0 $r2 255
cmp $r0 0xFFFF
cmp $r1 $r2
push $r0
push 0xFFFF
pop $r1
sw $r0 $r1
mov $a1 0xFF
jeq $r0
jlt 512
jne $r2
";

        let mut buffer = [0u8; 3];
        let mut binary_instr = 0u32;

        let parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        let binary_parsed_instructions = parsed_instructions
            .iter()
            .map(|x| {
                x.to_binary(&mut buffer);
                buffer_to_u32(&buffer)
            })
            .collect::<Vec<_>>();

        fn buffer_to_u32(buffer: &[u8; 3]) -> u32 {
            (buffer[0] as u32) << 16 | (buffer[1] as u32) << 8 | (buffer[2] as u32)
        }

        // opcode-type-padding-reg_a-reg_b-imm
        expected_instructions.push(0b0000_1_00000_000_001_00000001); // add $r0 $r1 1
        expected_instructions.push(0b0001_0_0000000000_100_101_001); // sub $a1 $a2 $r1
        expected_instructions.push(0b0010_0_0000000000_101_101_110); // and $a2 $a2 $z
        expected_instructions.push(0b0110_1_00000_011_010_11111111); // asr $a0 $r2 255
        expected_instructions.push(0b0111_1_000___1111111111111111); // cmp $r0 0xFF
        expected_instructions.push(0b0111_0_0000000000000__001_010); // cmp $r1 $r2
        expected_instructions.push(0b1000_0_0000000000000000___000); // push $r0
        expected_instructions.push(0b1000_1_000___1111111111111111); // push 0xFFFF
        expected_instructions.push(0b1001_0_0000000000000000___001); // pop $r1
        expected_instructions.push(0b1011_0_0000000000000__000_001); // sw $r0 $r1
        expected_instructions.push(0b1100_0___100_0000000011111111); // mov $a1 0xFF
        expected_instructions.push(0b1101_0_0000000000000000___000); // jeq $r0
        expected_instructions.push(0b1110_1_000___0000001000000000); // jlt 512
        expected_instructions.push(0b1111_0_0000000000000000___010); // jne $r2

        // print binary numbers

        for i in 0..expected_instructions.len() {
            println!("Instruction: {:?}", parsed_instructions[i]);
            println!("Got:      {:0>32b}", binary_parsed_instructions[i]);
            println!("Expected: {:0>32b}", expected_instructions[i]);
            assert_eq!(binary_parsed_instructions[i], expected_instructions[i]);
        }

        let decompiled_instructions = binary_parsed_instructions
            .iter()
            .map(|&x| Instruction::from_binary(x))
            .collect::<Vec<_>>();

        assert_sequence(&decompiled_instructions, &parsed_instructions);
    }

    #[test]
    fn parsing_errors() {
        let code = "add $r0 $r1 290 # outside 8 bit constraints";
    }
}
