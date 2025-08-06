#[derive(Clone)]
pub(crate) struct Logger(Rc<RefCell<Vec<String>>>);

impl Logger {
    pub(super) fn new() -> Self {
        Logger(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let string = match level {
            Info => format!("[INFO] {}", message),
            Warning => format!("[WARNING] {}", message),
            Error => format!("[ERROR] {}", message),
        };
        self.0.borrow_mut().push(string);
    }

    pub(super) fn get_logs(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.0.borrow()
    }
}

pub struct Emulator {
    screen: String,
    screen_modified: bool,
    text: Vec<u8>,
    screen_width: u8,
    screen_height: u8,
    logger: Logger,
}

pub(crate) enum LogLevel {
    Info,
    Warning,
    Error,
}

use std::cell::RefCell;
use std::{ptr::NonNull, rc::Rc};

use LogLevel::*;
use crossterm::event::KeyEventKind;

impl Emulator {
    pub(super) fn new(width: u8, height: u8, logger: Logger) -> Self {
        Emulator {
            screen_modified: false,
            screen_width: width,
            screen_height: height,
            screen: String::new(),
            text: vec![' '.to_ascii_lowercase() as u8; (width * height) as usize],
            logger,
        }
    }

    pub(super) fn screen(&mut self) -> &str {
        if self.screen_modified {
            let mut screen_lines = self
                .screen
                .lines()
                .map(|l| l.to_owned())
                .collect::<Vec<_>>();

            for text_line_idx in 0..self.screen_height {
                let text_line = &self.text[text_line_idx as usize * self.screen_width as usize
                    ..(text_line_idx + 1) as usize * self.screen_width as usize];
                let mut screen_line = String::from("║  ");
                for i in 0..self.screen_width {
                    let c = text_line[i as usize] as char;
                    screen_line.push(c);
                }
                screen_line.push_str("  ║");
                screen_lines[(text_line_idx + 1) as usize] = screen_line;
            }

            let final_lines = screen_lines
                .iter()
                .fold(String::new(), |acc, line| acc + line + "\n");
            self.screen = final_lines;
        }
        &self.screen
    }

    pub fn clear_screen(&mut self) {
        let top_border = "╔".to_owned()
            + &String::from_iter(std::iter::repeat('═').take(4 + self.screen_width as usize))
            + "╗";
        let bottom_border = "╚".to_owned()
            + &String::from_iter(std::iter::repeat('═').take(4 + self.screen_width as usize))
            + "╝";
        let line = String::from("║")
            + &String::from_iter(std::iter::repeat(' ').take(4 + self.screen_width as usize))
            + "║";

        self.screen.clear();
        self.screen.push_str(&top_border);
        self.screen.push('\n');
        for _ in 0..self.screen_height {
            self.screen.push_str(&line);
            self.screen.push('\n');
        }
        self.screen.push_str(&bottom_border);
        self.screen.push('\n');

        self.text =
            vec![' '.to_ascii_lowercase() as u8; (self.screen_width * self.screen_height) as usize];
    }

    pub fn set_char(&mut self, x: u8, y: u8, c: char) {
        if x >= self.screen_width || y >= self.screen_height {
            self.logger.log(Error, "Coordinates out of bounds");
            return;
        }
        if !c.is_ascii() {
            self.logger.log(Error, "Character is not ASCII");
            return;
        }
        self.text[(y as usize * self.screen_width as usize + x as usize) as usize] = c as u8;
        let text = self.text.iter().map(|&b| b as char).collect::<String>();
        self.logger
            .log(Info, &format!("Setting character {} at ({}, {})", c, x, y));
        self.logger
            .log(Info, &format!("Current text in emulator: {}", text));
        self.screen_modified = true;
    }
}

pub struct Display {
    ctrl: NonNull<u8>,
    data: NonNull<u8>,
    cursor_position: (u8, u8),
    logger: Logger,
}

impl Display {
    // Pins
    pub const DISPLAY_ENABLE: u8 = 0b1000_0000;
    pub const DISPLAY_WRITE: u8 = 0b0100_0000;
    pub const DISPLAY_CURSOR_MOVE: u8 = 0b0010_0000; // TODO: remove
    pub const DISPLAY_CLEAR_OR_CRESET: u8 = 0b0001_0000;

    // Cursor helpers
    pub const CURSOR_MASK: u8 = 0b0000_1100;
    pub const CURSOR_UP: u8 = 0b0000_1000;
    pub const CURSOR_DOWN: u8 = 0b0000_0000;
    pub const CURSOR_LEFT: u8 = 0b0000_0100;
    pub const CURSOR_RIGHT: u8 = 0b0000_1100;

    pub fn new(data: *const u8, ctrl: *const u8, logger: Logger) -> Self {
        Display {
            ctrl: NonNull::new(ctrl as *mut u8).unwrap(),
            data: NonNull::new(data as *mut u8).unwrap(),
            cursor_position: (0, 0),
            logger,
        }
    }

    pub fn update(&mut self, emulator: &mut Emulator) {
        let ctrl = unsafe { self.ctrl.read() };
        let data = unsafe { self.data.read() };

        if ctrl & Self::DISPLAY_ENABLE != 0 {
            self.logger.log(LogLevel::Info, "Display enabled");
            if ctrl & Self::DISPLAY_WRITE != 0 {
                let (x, y) = self.cursor_position;
                let c = data as char;
                emulator.set_char(x, y, c);
            } else if ctrl & Self::DISPLAY_CURSOR_MOVE == 0 {
                // No cursor move
                if ctrl & Self::DISPLAY_CLEAR_OR_CRESET != 0 {
                    // Clear screen
                    self.logger.log(LogLevel::Info, "Clearing screen");
                    emulator.clear_screen();
                } else {
                    // Cursor reset
                    self.logger.log(LogLevel::Info, "Resetting cursor position");
                    self.cursor_position = (0, 0);
                }
            }
            let move_cmd = ctrl & Self::CURSOR_MASK;
            if move_cmd != 0 {
                // Cursor move
                match move_cmd {
                    Self::CURSOR_UP => self.cursor_position.1 += 1,
                    Self::CURSOR_DOWN => self.cursor_position.1 -= 1,
                    Self::CURSOR_LEFT => self.cursor_position.0 -= 1,
                    Self::CURSOR_RIGHT => self.cursor_position.0 += 1,
                    _ => {
                        unreachable!();
                    }
                }
                self.logger.log(
                    LogLevel::Info,
                    &format!(
                        "Cursor move command: {:#04x}, new cursor position: {:?}",
                        move_cmd, self.cursor_position
                    ),
                );
            }
        }
    }
}

pub struct Keyboard {
    keycode: u8,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard { keycode: 0 }
    }

    pub fn poll(&mut self) -> bool {
        use crossterm::event::{self, Event, KeyCode, KeyEvent};

        if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(100)) {
            if let Ok(Event::Key(KeyEvent { code, kind, .. })) = event::read() {
                const IS_CHAR: u8 = 0b0100_0000;
                let key_code = match code {
                    KeyCode::Char(c) => {
                        let c_code = c.to_ascii_uppercase() as u8 - ' ' as u8; // should be between 0 and 64
                        if c_code < 64 { c_code | IS_CHAR } else { 0 }
                    }
                    KeyCode::Up => 0b0000_0100,        // Up arrow
                    KeyCode::Down => 0b0000_0101,      // Down arrow
                    KeyCode::Left => 0b0000_0110,      // Left arrow
                    KeyCode::Right => 0b0000_0111,     // Right arrow
                    KeyCode::Enter => 0b0000_0001,     // Enter key
                    KeyCode::Backspace => 0b0000_0010, // Backspace key
                    KeyCode::Esc => 0b0000_0011,       // Escape key
                    _ => 0,
                };
                if key_code != 0 {
                    let key_state = if let KeyEventKind::Press = kind {
                        0b1000_0000 // Key pressed
                    } else {
                        0b0000_0000 // Key released
                    };

                    let keycode = key_code | key_state;
                    self.keycode = keycode;
                    return true;
                }
            }
        }
        return false; // No key pressed
    }
}
