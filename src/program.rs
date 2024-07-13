use std::process::{Child, Command, Stdio};

use crate::config::TMProgramConfig;

#[derive(Debug)]
pub enum ProgramStatus {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Paging,
    Unknown,
}

impl Default for ProgramStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct TMProgram {
    pub config: TMProgramConfig,
    pub child: Option<Child>,
}

impl TMProgram {
    pub fn launch(&mut self) {
        let program: Vec<&str> = self.config.command.split(' ').collect();
        let cmd = program[0];
        let args = &program[1..program.len()];
        match Command::new(cmd).args(args).stdout(Stdio::piped()).spawn() {
            Ok(x) => self.child = Some(x),
            Err(e) => eprintln!("failed to spawn program: {}", e),
        }
    }
}
