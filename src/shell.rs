use std::io::{self, Read, Write};
use std::process::Command;

pub struct Shell {
    history: Vec<String>,
    history_index: usize,
    current_input: String,
}

impl Shell {
    fn new() -> Self {
        Shell {
            history: Vec::new(),
            history_index: 0,
            current_input: String::new(),
        }
    }

    fn run(&mut self) -> io::Result<()> {
        // Set terminal to raw mode to capture arrow keys
        let original_termios = self.set_raw_mode()?;

        loop {
            print!("> ");
            io::stdout().flush()?;

            let input = self.read_line()?;
            if input.trim() == "exit" {
                break;
            }

            if !input.trim().is_empty() {
                // Add command to history only if non-empty
                self.history.push(input.clone());
                self.history_index = self.history.len();

                // Execute command
                self.execute_command(&input)?;
            }
        }

        // Restore terminal settings
        self.restore_terminal(original_termios)?;
        Ok(())
    }

    fn set_raw_mode(&self) -> io::Result<libc::termios> {
        unsafe {
            let mut termios = std::mem::zeroed();
            if libc::tcgetattr(0, &mut termios) != 0 {
                return Err(io::Error::last_os_error());
            }

            let original_termios = termios;

            // Disable canonical mode and echo
            termios.c_lflag &= !(libc::ICANON | libc::ECHO);
            // Set min characters and timeout
            termios.c_cc[libc::VMIN] = 1;
            termios.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(0, libc::TCSANOW, &termios) != 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(original_termios)
        }
    }

    fn restore_terminal(&self, termios: libc::termios) -> io::Result<()> {
        unsafe {
            if libc::tcsetattr(0, libc::TCSANOW, &termios) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn read_line(&mut self) -> io::Result<String> {
        let mut input = String::new();
        let mut cursor_pos = 0;
        self.current_input = String::new();

        loop {
            let mut buf = [0; 1];
            io::stdin().read_exact(&mut buf)?;

            match buf[0] {
                b'\n' | b'\r' => {
                    println!();
                    break;
                }
                127 | 8 => {
                    // Backspace
                    if cursor_pos > 0 {
                        input.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                        // Move cursor back and clear character
                        print!("\x08 \x08");
                        io::stdout().flush()?;
                    }
                }
                27 => {
                    // Escape sequence (arrow keys)
                    let mut seq = [0; 2];
                    io::stdin().read_exact(&mut seq)?;

                    if seq[0] == b'[' {
                        match seq[1] {
                            b'A' => {
                                // Up arrow
                                if !self.history.is_empty() {
                                    // Save current input if first time pressing up
                                    if self.history_index == self.history.len() {
                                        self.current_input = input.clone();
                                    }

                                    if self.history_index > 0 {
                                        self.history_index -= 1;
                                        // Clear current line
                                        self.clear_line(&input, cursor_pos)?;

                                        input = self.history[self.history_index].clone();
                                        cursor_pos = input.len();
                                        print!("{}", input);
                                        io::stdout().flush()?;
                                    }
                                }
                            }
                            b'B' => {
                                // Down arrow
                                if self.history_index < self.history.len() {
                                    self.history_index += 1;
                                    // Clear current line
                                    self.clear_line(&input, cursor_pos)?;

                                    if self.history_index == self.history.len() {
                                        input = self.current_input.clone();
                                    } else {
                                        input = self.history[self.history_index].clone();
                                    }

                                    cursor_pos = input.len();
                                    print!("{}", input);
                                    io::stdout().flush()?;
                                }
                            }
                            _ => {} // Ignore other escape sequences
                        }
                    }
                }
                // Regular character
                c => {
                    input.insert(cursor_pos, c as char);
                    cursor_pos += 1;
                    print!("{}", c as char);
                    io::stdout().flush()?;
                }
            }
        }

        Ok(input)
    }

    fn clear_line(&self, input: &str, cursor_pos: usize) -> io::Result<()> {
        // Move cursor to beginning of line and clear to end
        print!("\r> ");
        for _ in 0..input.len() {
            print!(" ");
        }
        print!("\r> ");
        io::stdout().flush()?;
        Ok(())
    }

    fn execute_command(&self, command: &str) -> io::Result<()> {
        let mut parts = command.trim().split_whitespace();
        let command = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();

        match Command::new(command).args(&args).spawn() {
            Ok(mut child) => {
                child.wait()?;
                Ok(())
            }
            Err(e) => {
                println!("Error: {}", e);
                Ok(())
            }
        }
    }
}
