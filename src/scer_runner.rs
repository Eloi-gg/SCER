use std::env;
use std::io::{Read, Write};

use machine::Machine;
use argh::FromArgs;

mod emulator;
mod machine;
mod program;
mod utils;

struct Args {
    /// path to the input file (.sp or .csp)
    #[argh(positional)]
    file_path: String,

    /// enable debug mode
    #[argh(switch, short = 'd')]
    debug: bool,


}

fn display_logo() {
    println!(
        "

       ▐▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▌
       ▐        ███████╗ ██████╗███████╗██████╗         ▌
       ▐        ██╔════╝██╔════╝██╔════╝██╔══██╗        ▌
       ▐        ███████╗██║     █████╗  ██████╔╝        ▌
       ▐        ╚════██║██║     ██╔══╝  ██╔══██╗        ▌
       ▐███████╗███████║╚██████╗███████╗██║  ██║███████╗▌
       ▐╚══════╝╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═╝╚══════╝▌
       ▐▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▌

    ░█▀▀░█▄█░█▀█░█░░░█░░░░░█▀▀░█▀█░█▄█░█▀█░█░█░▀█▀░█▀▀░█▀▄
    ░▀▀█░█░█░█▀█░█░░░█░░░░░█░░░█░█░█░█░█▀▀░█░█░░█░░█▀▀░█▀▄
    ░▀▀▀░▀░▀░▀░▀░▀▀▀░▀▀▀░░░▀▀▀░▀▀▀░▀░▀░▀░░░▀▀▀░░▀░░▀▀▀░▀░▀
    ░█▀▀░█▄█░█░█░█░░░█▀█░▀█▀░█▀▀░█▀▄░░░▀█▀░█▀█░░░█▀▄░█░█░█▀▀░▀█▀
    ░█▀▀░█░█░█░█░█░░░█▀█░░█░░█▀▀░█░█░░░░█░░█░█░░░█▀▄░█░█░▀▀█░░█░
    ░▀▀▀░▀░▀░▀▀▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀▀░░░░▀▀▀░▀░▀░░░▀░▀░▀▀▀░▀▀▀░░▀░

    "
    );
}

fn load_args() -> (String, bool) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file.sp> [-d]", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let debug_mode = args.contains(&String::from("-d"));

    if !file_path.ends_with(".sp") && !file_path.ends_with(".csp") {
        eprintln!("Error: The file must have a .sp or .csp extension.");
        std::process::exit(1);
    }

    if debug_mode {
        println!("Debug mode is enabled.");
        (file_path.to_string(), true)
    } else {
        println!("Debug mode is disabled.");
        (file_path.to_string(), false)
    }
}

fn save_binary(program_path: &str, data: &[u8]) {
    let program_path = std::path::Path::new(program_path);
    let dir_path = program_path
        .parent()
        .unwrap()
        .join(std::path::Path::new("/compiled_programs/"));
    std::fs::create_dir(dir_path.clone()).expect("Could not create directory");

    let mut file = std::fs::File::create(
        dir_path.join(
            program_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .replace(".sp", ".csp"),
        ),
    )
        .expect("Could not create file");
    file.write_all(data).expect("Could not write to file");
}

fn clr() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    display_logo();
    let (file_path, debug_mode) = load_args();
    let program_name = file_path.split("/").last().unwrap().to_string();

    let program = if file_path.ends_with(".sp") {
        program::ScarProgram::compile(
            std::fs::read_to_string(file_path).expect("Could not read file"),
        )
            .expect("Could not compile program.")
    } else {
        std::fs::read(file_path).expect("Could not read file")
    };
    save_binary(&program_name, &program);

    println!("Program: {}\nPress enter to start...", program_name);
    let mut buffer = [0u8; 8];
    let _ = std::io::stdin().read(&mut buffer).unwrap();
    clr();

    let mut emulator = emulator::Emulator::new(16, 2);
    let mut machine = Machine::new();
    emulator.clear_screen();
    print!("{}\n", emulator.screen());
    print!("{}", machine.get_state());

    // TODO : hook display

    if debug_mode {
        for _ in 0..10 {
            let kb_in_len: usize = std::io::stdin().read(&mut buffer).unwrap();
            machine.step();
            clr();
            // TODO: update display
            print!("{}\n", emulator.screen());
            print!("{}", machine.get_state());
        }
    }
    println!();

    // Add your logic to process the .sp file here
}
