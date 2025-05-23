use std::fmt;
use std::fmt::{Display, Formatter};

use crate::program::TMProgram;

#[derive(Debug)]
#[cfg(target_os = "linux")]
pub enum ProgramState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    TracingStop,
    Dead,
}

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "macos")]
pub enum ProgramState {
    ProcessCreated,
    Running,
    Sleeping,
    Zombie,
}

#[cfg(target_os = "macos")]
impl TryFrom<u32> for ProgramState {
    type Error = StateError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ProcessCreated),
            2 => Ok(Self::Running),
            3 => Ok(Self::Sleeping),
            4 => Ok(Self::Zombie),
            _ => Err(StateError::UnknownState),
        }
    }
}

#[derive(Debug)]
pub enum StateError {
    UnknownState,
    ProgramNotLaunched,
}
impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TMProgram {
    #[cfg(target_os = "linux")]
    pub fn state(&mut self) -> Result<ProgramState, StateError> {
        use std::fs::File;
        use std::io::Read;
        let path_state = match &self.child {
            None => return Err(StateError::ProgramNotLaunched),
            Some(x) => format!("/proc/{}/status", x.id()),
        };
        let mut file = match File::open(path_state) {
            Err(e) => return Err(StateError::RuntimeError(Box::new(e))),
            Ok(x) => x,
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

    #[cfg(target_os = "macos")]
    pub fn state(&mut self) -> Result<ProgramState, StateError> {
        use libc::{c_void, proc_bsdinfo, proc_pidinfo, PROC_PIDTBSDINFO};
        use std::mem;

        let proc_info = match &self.child {
            None => return Err(StateError::ProgramNotLaunched),
            Some(x) => {
                let mut proc_info: proc_bsdinfo = unsafe { mem::zeroed() };
                unsafe {
                    proc_pidinfo(
                        x.id() as i32,
                        PROC_PIDTBSDINFO,
                        0,
                        &mut proc_info as *mut _ as *mut c_void,
                        size_of::<proc_bsdinfo>() as i32,
                    );
                };
                proc_info
            }
        };
        proc_info.pbi_status.try_into()
    }
}
