use std::env;
use std::io::{Read, Write};

use crate::emulator::LogLevel;
use cgi::KeyCode;
use machine::Machine;

mod emulator;
mod machine;
mod program;
mod ui;

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

static CONSOLE: once_cell::sync::Lazy<
    std::sync::Mutex<cgi::debug::dbg_window::connect::DebugConsole>,
> = once_cell::sync::Lazy::new(|| {
    const CONNECTION_NAME: &str = "app log";
    const CONNECTION_IP: &str = "127.0.0.2";
    const CONNECTION_PORT: u16 = 4000;

    cgi::debug::dbg_window::create::spawn_server(
        cgi::log::get_dbg_window_exe_path().to_str().unwrap(),
        CONNECTION_IP,
        CONNECTION_PORT,
    );
    let mut logger = cgi::debug::dbg_window::connect::connect_to_server(
        CONNECTION_NAME,
        CONNECTION_IP,
        CONNECTION_PORT,
        Some(std::time::Duration::from_secs(5)),
    )
    .unwrap();
    logger.send_message("APP CONNECTED");
    std::sync::Mutex::new(logger)
});

fn console_println(string: &str) {
    let mut console = CONSOLE.try_lock().unwrap();
    console.send_message(string);
}

fn main() {
    use emulator::LogLevel::*;
    const NUM_OLD_MESSAGES: usize = 8;

    display_logo();

    // Load arguments
    let (file_path, debug_mode) = load_args();
    let program_name = file_path.split("/").last().unwrap().to_string();
    let program = std::fs::read(file_path).expect("Could not read file");

    // Wait for user input to start
    println!("Program: {}\nPress enter to start...", program_name);
    let mut buffer = [0u8; 8];
    let _ = std::io::stdin().read(&mut buffer).unwrap();

    let (mut app, app_connection) = cgi::Application::new();
    let mut ui = ui::ScerUi::new();
    ui.add_all_layouts(&mut app);
    app.spawn_debug_window();
    console_println("App start");
    let mut event_receiver = app.add_event_receiver();
    // todo!("use event_receiver");
    //
    app_connection.send_command(cgi::Command::FocusWidget(ui.test()));
    std::thread::spawn(move || {
        // Initialize emulator and machine
        let logger = emulator::Logger::new();
        logger.log(Info, "Starting SCER Runner..."); //TODO: switch to a cgi logger
        let mut display = emulator::Display::new(logger.clone());
        // let mut keyboard = emulator::Keyboard::new(logger.clone(), debug_mode);
        let mut emulator = emulator::Emulator::new(16, 2, logger.clone());
        let mut machine = Machine::new();

        machine.load(&program);

        emulator.clear_screen();

        // TODO : hook display
        machine.add_peripheral(Machine::DISPLAY_CTRL_ADDR as u16, display.ctrl_addr());
        machine.add_peripheral(Machine::DISPLAY_DATA_ADDR as u16, display.data_addr());

        // TODO remove this
        machine.set_memory(0xF000, 10);

        let mut handle_events = |last_num_msg: &mut usize| {
            let n_returns = event_receiver
                .get_events_blocking()
                .filter(|e| matches!(e, cgi::Event::KeyPress(KeyCode::Enter)))
                .count();

            if n_returns == 0 {
                return;
            }

            for _ in 0..n_returns {
                machine.step();
            }

            display.update(&mut emulator);
            let logger = logger.get_logs();
            let log_len = logger.len();
            let old_messages = logger[last_num_msg.saturating_sub(NUM_OLD_MESSAGES)..*last_num_msg]
                .iter()
                .filter(|msg| !msg.is_empty())
                .collect::<Vec<_>>();
            let new_messages = logger[*last_num_msg..log_len]
                .iter()
                .filter(|msg| !msg.is_empty())
                .collect::<Vec<_>>();
            *last_num_msg = log_len;

            ui.update(&machine.state(), emulator.set_chars(), new_messages);
            app_connection.send_action(cgi::Action::RedrawAll);
            //TODO: link to UI
        };

        let mut last_num_msg: usize = 0;
        loop {
            handle_events(&mut last_num_msg);
        }
    });

    app.run();
}
