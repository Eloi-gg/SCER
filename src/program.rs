pub(crate) struct ScarProgram {
    instructions: Vec<Instruction>,
}

/// Instruction format (24 bits):
/// Arithmetic operations:
///     [op - 4 bits][type = 1][reg_d - 3 bits][reg_a - 3 bits][flag_enable - 1 bit][imm - 16 bits]
///     [op - 4 bits][type = 0][reg_d - 3 bits][reg_a - 3 bits][reg_b - 3 bits][flag_enable - 1 bit][todo]
/// Comparison operations:
///    [op - 4 bits][type = 1][reg_a - 3 bits][imm - 16 bits]
///    [op - 4 bits][type = 0][reg_a - 3 bits][reg_b - 3 bits]
/// Stack operations
///     [op - 4 bits][type = 1][imm - 16 bits]
///     [op - 4 bits][type = 0][reg_d - 3 bits]
/// Jump operations
///     [op - 4 bits][imm - 16 bits]
///

#[derive(Debug, PartialEq)]
enum Register {
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

#[derive(Debug, PartialEq)]
enum ArithmeticOp {
    Immediate {
        dest: Register,
        reg_a: Register,
        imm: u16,
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
            return Err(ParsingError::TooManyArguments);
        }
        let dest = Register::try_from(parts[0])?; // Dest must be a register
        let reg_a = Register::try_from(parts[1])?; // Reg A must be a register

        return match Register::try_from(parts[2]) {
            Ok(reg_b) => Ok(ArithmeticOp::Register { dest, reg_a, reg_b }),
            Err(e) => match e {
                ParsingError::IsNotRegister(_) => match try_immediate(parts[2]) {
                    Ok(imm) => Ok(ArithmeticOp::Immediate { dest, reg_a, imm }),
                    Err(_) => Err(ParsingError::InvalidInstruction),
                },
                _ => Err(e),
            },
        };
    }
}

impl TwoArgsOp {
    fn try_new(parts: &Vec<&str>) -> Result<TwoArgsOp, ParsingError> {
        if parts.len() != 2 {
            return Err(ParsingError::TooManyArguments);
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
}

#[derive(Debug, PartialEq)]
enum TwoArgsOp {
    Immediate(Register, u16),
    Register(Register, Register),
}

#[derive(Debug, PartialEq)]
enum OtherOp {
    Immediate(u16),
    Register(Register),
}

#[derive(Debug, PartialEq)]
pub enum ParsingError {
    EmptyLine,
    IsNotRegister(String),
    InvalidInstruction,
    TooManyArguments,
    InvalidRegister,
    UnknownInstruction(String),
}

#[derive(Debug, PartialEq)]
enum Instruction {
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
        if matches!(op, "push" | "jeq" | "jlt") {
            if args.len() != 1 {
                return Err(ParsingError::TooManyArguments);
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
                    },
                    _ => Err(e),
                }
            }?;

            return match op {
                "push" => Ok(Instruction::Push(op_content)),
                "jeq" => Ok(Instruction::Jeq(op_content)),
                "jlt" => Ok(Instruction::Jlt(op_content)),
                _ => unreachable!(),
            };
        }

        if "pop" == op {
            if args.len() != 1 {
                return Err(ParsingError::TooManyArguments);
            }
            return match Register::try_from(args[0]) {
                Ok(reg) => Ok(Instruction::Pop(reg)),
                Err(e) => Err(e),
            };
        }

        // Memory operations
        if matches!(op, "lw" | "sw") {
            if args.len() != 2 {
                return Err(ParsingError::TooManyArguments);
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
                return Err(ParsingError::TooManyArguments);
            }
            let reg = Register::try_from(args[0])?;
            let imm = try_immediate(args[1])?;
            return Ok(Instruction::Mov(reg, imm));
        }

        // Unknown instruction
        Err(ParsingError::UnknownInstruction(op.to_owned()))
    }

    fn to_binary(&self, buffer: &mut [u8]) {}
}

impl ScarProgram {
    pub fn compile(code: &str) -> Result<Vec<u8>, ParsingError> {
        let mut out = Vec::new();

        let mut buffer = [0u8; 3]; // 24bits per instruction
        for line in code.lines() {
            let instruction = Instruction::parse(line)?;
            instruction.to_binary(&mut buffer);
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
        let mut parsed_instructions = Vec::new();
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
or $r0 $f 12345
xor $r1 $a2 0xFFFF
asl $r2 $a0 1
asr $z $z 4
# comment c
";

        let mut parsed_instructions = parse_instructions(&code);
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
            imm: 12345,
        }));
        expected_instructions.push(Instruction::Xor(ArithmeticOp::Immediate {
            dest: Register::R1,
            reg_a: Register::A2,
            imm: 0xFFFF,
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

        let mut parsed_instructions = parse_instructions(&code);
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

        let mut parsed_instructions = parse_instructions(&code);
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

        let mut parsed_instructions = parse_instructions(&code);
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

        let mut parsed_instructions = parse_instructions(&code);
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

        let mut parsed_instructions = parse_instructions(&code);
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
";
        let mut parsed_instructions = parse_instructions(&code);
        let mut expected_instructions = Vec::new();

        expected_instructions.push(Instruction::Jeq(OtherOp::Immediate(1)));
        expected_instructions.push(Instruction::Jeq(OtherOp::Immediate(0x1234)));
        expected_instructions.push(Instruction::Jeq(OtherOp::Register(Register::R2)));
        expected_instructions.push(Instruction::Jlt(OtherOp::Immediate(0x0255)));
        expected_instructions.push(Instruction::Jlt(OtherOp::Register(Register::A0)));

        assert_sequence(&parsed_instructions, &expected_instructions);

        println!("{}\n", code);
        for instruction in parsed_instructions.iter() {
            println!("{:?}", instruction);
        }
    }


    #[test]
    fn defines() {}

    #[test]
    fn labels() {}

    #[test]
    fn parsing_errors() {}
}
