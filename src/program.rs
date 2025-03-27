pub(crate) struct ScarProgram {
    instructions: Vec<Instruction>,
}

enum Instruction {
    Nop
}

impl Instruction {
    fn parse(line: &str) -> Result<Instruction, String> {
        Ok(Instruction::Nop)
    }
}

impl ScarProgram {
    pub fn compile(code: &str) -> Result<ScarProgram, String> {
        let mut instructions = Vec::new();
        for line in code.lines() {
            let instruction = Instruction::parse(line)?;
            instructions.push(instruction);
        }
        Ok(ScarProgram { instructions })
    }

    pub fn to_machine_code(&self) -> Vec<u8> {
        let mut machine_code = Vec::new();
        for instruction in &self.instructions {
            match instruction {
                Instruction::Nop => machine_code.push(0),
            }
        }
        machine_code
    }
}
