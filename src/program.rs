use std::io;
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
    pub fn launch(&mut self) -> io::Result<()> {
        match Command::new(&self.config.command)
            .args(&self.config.args)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(x) => {
                self.child = Some(x);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
