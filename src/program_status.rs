use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::os::unix::prelude::ExitStatusExt;

use crate::program::TMProgram;
use crate::program_state::{ProgramState, StateError};

#[derive(Debug)]
pub enum ProgramStatus {
    Code(i32),             //exited
    Signal(i32),           //exited
    Running(ProgramState), // running
    Nothing,               //not launched
}

#[derive(Debug)]
pub enum StatusError {
    TryWaitFailed(io::Error),
    StateError(StateError),
    RuntimeError,
}

impl Display for StatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<StateError> for StatusError {
    fn from(value: StateError) -> Self {
        Self::StateError(value)
    }
}

impl Error for StatusError {}

impl TMProgram {
    pub fn status(&mut self) -> Result<ProgramStatus, StatusError> {
        match self.child.as_mut() {
            None => Ok(ProgramStatus::Nothing),
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => match status.code() {
                    Some(code) => Ok(ProgramStatus::Code(code)),
                    None => match status.signal() {
                        Some(signal) => Ok(ProgramStatus::Signal(signal)),
                        None => Err(StatusError::RuntimeError),
                    },
                },
                Ok(None) => Ok(ProgramStatus::Running(self.state()?)),
                Err(e) => Err(StatusError::TryWaitFailed(e)),
            },
        }
    }
}
