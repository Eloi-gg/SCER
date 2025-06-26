use std::env;
use std::io::{Read, Write};

use machine::Machine;


mod emulator;
mod machine;
mod program;
mod utils;

#[derive(argh::FromArgs)]
/// Scer Runner - A simple SCER program runner
struct Args {
    /// path to the input file (.csp)
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

    if !file_path.ends_with(".csp") {
        eprintln!("Error: The file must have a .csp extension.");
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

fn clr() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    display_logo();
    let (file_path, debug_mode) = load_args();
    let program_name = file_path.split("/").last().unwrap().to_string();

    let program = std::fs::read(file_path).expect("Could not read file");

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
}
