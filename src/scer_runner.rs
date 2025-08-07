use std::env;
use std::io::{Read, Write};

use machine::Machine;

mod emulator;
mod machine;
mod program;

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
    use emulator::LogLevel::*;

    display_logo();

    // Load arguments
    let (file_path, debug_mode) = load_args();
    let program_name = file_path.split("/").last().unwrap().to_string();
    let program = std::fs::read(file_path).expect("Could not read file");

    // Wait for user input to start
    println!("Program: {}\nPress enter to start...", program_name);
    let mut buffer = [0u8; 8];
    let _ = std::io::stdin().read(&mut buffer).unwrap();
    clr();

    // Initialize emulator and machine
    let logger = emulator::Logger::new();
    logger.log(Info, "Starting SCER Runner...");
    let mut display = emulator::Display::new(logger.clone());
    let mut keyboard = emulator::Keyboard::new(logger.clone());
    let mut emulator = emulator::Emulator::new(16, 2, logger.clone());
    let mut machine = Machine::new();
    machine.load(&program);

    emulator.clear_screen();

    // TODO : hook display
    // TODO remove this
    machine.set_memory(0xF000, 10);

    if debug_mode {
        const NUM_OLD_MESSAGES: usize = 8;
        let mut last_num_msg: usize = 0;
        print!("{}\n", emulator.screen());
        print!("{:?}", machine);

        let _kb_in_len: usize = std::io::stdin().read(&mut buffer).unwrap();

        for _step in 0.. {
            machine.step();
            display.update(&mut emulator);

            if machine.get_memory(Machine::MEMORY_END.try_into().unwrap()) != 0 {
                println!("Program finished.");
                break;
            }

            clr();
            print!("{}\n", emulator.screen());
            print!("{:?}", machine);

            let logger = logger.get_logs();

            // Log old messages
            println!("Old messages:");
            let log_len = logger.len();
            for msg in logger[last_num_msg.saturating_sub(NUM_OLD_MESSAGES)..last_num_msg].iter() {
                if !msg.is_empty() {
                    println!("{}", msg);
                }
            }
            println!("-------------------------------");

            // Log new messages
            println!("New messages:");
            if last_num_msg != log_len {
                for msg in logger[last_num_msg..log_len].iter() {
                    if !msg.is_empty() {
                        println!("{}", msg);
                    }
                }
                last_num_msg = log_len;
            }
            println!("-------------------------------");

            let _kb_in_len: usize = std::io::stdin().read(&mut buffer).unwrap();
        }
    } else {
        for _step in 0.. {
            machine.step();
            display.update(&mut emulator);
            clr();
            print!("{}\n", emulator.screen());
            if machine.get_memory(Machine::MEMORY_END.try_into().unwrap()) != 0 {
                println!("Program finished.");
                let _kb_in_len: usize = std::io::stdin().read(&mut buffer).unwrap();
                break;
            }
        }
    }
    println!();
}
