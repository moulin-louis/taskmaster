use std::fmt::Display;

use tokio::io::{self, stdin, AsyncBufReadExt};
use tokio::io::{AsyncWriteExt, Stdout};

pub struct Shell {
    shell: String,
    stdout: Stdout,
    og_termios: libc::termios,
}

#[derive(Debug)]
pub enum TryNewError {
    TcGetAttr,
}

impl Display for TryNewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::error::Error for TryNewError {}

#[allow(dead_code)]
impl Shell {
    pub fn try_new(shell: &str) -> Result<Self, TryNewError> {
        let stdout = tokio::io::stdout();
        let og_termios = match Shell::set_raw_mode() {
            Ok(x) => x,
            Err(_) => return Err(TryNewError::TcGetAttr),
        };
        let shell = Shell {
            og_termios,
            stdout,
            shell: shell.to_string(),
        };
        Ok(shell)
    }

    fn set_raw_mode() -> io::Result<libc::termios> {
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

    pub async fn read_line(&mut self) -> Result<String, io::Error> {
        self.stdout.write_all(b"$>").await?;
        self.stdout.flush().await?;
        let mut user_input = String::new();
        let mut reader = io::BufReader::new(stdin());
        reader.read_line(&mut user_input).await?;
        Ok(user_input)
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.og_termios);
        }
    }
}
