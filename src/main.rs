use std::env;
use std::io::Read;

mod emulator;
mod program;
mod utils;
mod machine;

fn display_logo() {
    println!("

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
    
    ");
}

fn load_args() -> (String, bool) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file.sp> [-d]", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let debug_mode = args.contains(&String::from("-d"));

    if !file_path.ends_with(".sp") {
        eprintln!("Error: The file must have a .sp extension.");
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



fn main() {
    display_logo();
    let (file_path, debug_mode) = load_args();
    let program_name = file_path.split("/").last().unwrap().to_string();
    let file_data = std::fs::read_to_string(file_path).expect("Could not read file");
    let program = program::ScarProgram::compile(&file_data).unwrap();

    println!("Program: {}\nPress enter to start...", program_name);
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();

    let mut emulator = emulator::Emulator::new(16, 2);
    emulator.clear_screen();
    print!("{}\n", emulator.screen());
    emulator.set_char(0, 0, 'E');
    print!("{}\n", emulator.screen());

    // Add your logic to process the .sp file here
}