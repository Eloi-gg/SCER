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
            let mut screen_lines = self.screen.lines().map(|l| l.to_owned()).collect::<Vec<_>>();

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

            let final_lines = screen_lines.iter().fold(String::new(), |acc, line| {
                acc + line + "\n"
            });
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

        self.text = vec![' '.to_ascii_lowercase() as u8; (self.screen_width * self.screen_height) as usize];
    }

    pub fn set_char(&mut self, x: usize, y: usize, c: char) {
        if x >= self.screen_width as usize || y >= self.screen_height as usize {
            Self::log(Error, "Coordinates out of bounds");
            return;
        }
        if !c.is_ascii() {
            Self::log(Error, "Character is not ASCII");
            return;
        }
        self.text[(y * self.screen_width as usize + x) as usize] = c as u8;
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

