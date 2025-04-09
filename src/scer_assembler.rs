use std::env;
use std::io::{Read, Write};
mod program;
use argh;

#[derive(argh::FromArgs)]
/// SCER Assembler
struct Args {
    /// assembly file path
    #[argh(positional)]
    file_path: String,

    /// output file path (optional)
    #[argh(option)]
    output_path: Option<String>,
}

fn save_binary(program_path: &str, output_path: &str, data: &[u8]) {
    let mut file = std::fs::File::create(output_path).expect("Could not create file");
    file.write_all(data).expect("Could not write to file");
}

fn clr() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    let args: Args = argh::from_env();

    let program_name = args.file_path.split("/").last().unwrap().to_string();
    let program_content = std::fs::read_to_string(&args.file_path).expect("Could not read file");
    let program = program::ScarProgram::compile(program_content).expect("Could not compile program.");
    let output_path = args.output_path.unwrap_or_else(|| {
        args.file_path
            .replace(".sp", ".csp")
    });

    save_binary(&program_name, &output_path, &program);
}
