pub struct Emulator {
    screen: String,
    screen_modified: bool,
    text: Vec<u8>,
    screen_width: u8,
    screen_height: u8,
}

enum LogLevel {
    Info,
    Warning,
    Error,
}

use std::ptr::NonNull;

use LogLevel::*;

impl Emulator {
    pub(super) fn new(width: u8, height: u8) -> Self {
        Emulator {
            screen_modified: false,
            screen_width: width,
            screen_height: height,
            screen: String::new(),
            text: vec![' '.to_ascii_lowercase() as u8; (width * height) as usize],
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
            Self::log(Error, "Coordinates out of bounds");
            return;
        }
        if !c.is_ascii() {
            Self::log(Error, "Character is not ASCII");
            return;
        }
        self.text[(y as usize * self.screen_width as usize + x as usize) as usize] = c as u8;
        self.screen_modified = true;
    }

    fn log(log_level: LogLevel, message: &str) {
        match log_level {
            LogLevel::Info => println!("[INFO] {}", message),
            LogLevel::Warning => println!("[WARNING] {}", message),
            LogLevel::Error => eprintln!("[ERROR] {}", message),
        }
    }
}

struct Display {
    ctrl: NonNull<u8>,
    data: NonNull<u8>,
    cursor_position: (u8, u8),
}

impl Display {
    // Pins
    const DISPLAY_ENABLE: u8 = 0b1000_0000;
    const DISPLAY_WRITE: u8 = 0b0100_0000;
    const DISPLAY_CLEAR: u8 = 0b0010_0000;
    const DISPLAY_CURSOR_MOVE: u8 = 0b0001_0000;

    // Cursor helpers
    const CURSOR_MASK: u8 = 0b0000_1100;
    const CURSOR_UP: u8 = 0b0000_1000;
    const CURSOR_DOWN: u8 = 0b0000_0000;
    const CURSOR_LEFT: u8 = 0b0000_0100;
    const CURSOR_RIGHT: u8 = 0b0000_1100;

    fn new(data: *const u8, ctrl: *const u8) -> Self {
        Display {
            ctrl: NonNull::new(ctrl as *mut u8).unwrap(),
            data: NonNull::new(ctrl as *mut u8).unwrap(),
            cursor_position: (0, 0),
        }
    }

    fn update(&mut self, emulator: &mut Emulator) {
        let ctrl = unsafe { self.ctrl.read() };
        let data = unsafe { self.data.read() };

        if ctrl & Self::DISPLAY_ENABLE != 0 {
            if ctrl & Self::DISPLAY_WRITE != 0 {
                let (x, y) = self.cursor_position;
                let c = data as char;
                emulator.set_char(x, y, c);
            } else if ctrl & Self::DISPLAY_CLEAR != 0 {
                emulator.clear_screen();
            } else if ctrl & Self::DISPLAY_CURSOR_MOVE != 0 {
                let move_cmd = ctrl & Self::CURSOR_MASK;
                match move_cmd {
                    Self::CURSOR_UP => self.cursor_position.1 += 1,
                    Self::CURSOR_DOWN => self.cursor_position.1 -= 1,
                    Self::CURSOR_LEFT => self.cursor_position.0 -= 1,
                    Self::CURSOR_RIGHT => self.cursor_position.0 += 1,
                    _ => {
                        unreachable!();
                    }
                }
            }
        }
    }
}
