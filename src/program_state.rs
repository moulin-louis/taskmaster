use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;

use crate::program::TMProgram;

#[derive(Debug)]
pub enum ProgramState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    TracingStop,
    Dead,
}

impl TryFrom<&str> for ProgramState {
    type Error = StateError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "R" => Ok(Self::Running),
            "S" => Ok(Self::Sleeping),
            "D" => Ok(Self::DiskSleep),
            "Z" => Ok(Self::Zombie),
            "T" => Ok(Self::TracingStop),
            "X" => Ok(Self::Dead),
            _ => Err(StateError::UnknownState(value.to_string())), // Capture the unknown state
        }
    }
}

#[derive(Debug)]
pub enum StateError {
    UnknownState(String),
    ProgramNotLaunched,
    RuntimeError(Box<dyn Error>),
}
impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TMProgram {
    pub fn state(&mut self) -> Result<ProgramState, StateError> {
        let path_state = match &self.child {
            None => return Err(StateError::ProgramNotLaunched),
            Some(x) => format!("/proc/{}/status", x.id()),
        };
        let mut file = match File::open(path_state) {
            Ok(x) => x,
            Err(e) => return Err(StateError::RuntimeError(Box::new(e))),
        };

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("cant read status file to end");
        let status_line = content.lines().nth(2);
        let status_line = match status_line {
            None => return Err(StateError::UnknownState("Missing State".to_string())),
            Some(x) => x,
        };
        let binding = status_line.chars().nth(7).unwrap().to_string();
        binding.as_str().try_into()
    }
}
