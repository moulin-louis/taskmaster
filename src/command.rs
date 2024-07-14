use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::program::ProgramStatus;
use crate::Ordering;
use crate::TMProgram;

#[derive(Debug)]
pub enum CommandUser {
    List,
    Kill(u32),
    Restart(u32),
    Launch(u32),
    Status(u32),
    Exit,
}

#[derive(Debug)]
pub enum CommandError {
    ProgramNotLaunched,
    FailedOpenFile,
    WrongIndex,
    RuntimeError,
    UnknownStatus,
    UnknownCommand,
    MissingParams,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CommandError {}

impl TryFrom<&str> for ProgramStatus {
    type Error = CommandError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains("sleeping") {
            Ok(ProgramStatus::Sleeping)
        } else if value.contains("running") {
            Ok(ProgramStatus::Running)
        } else if value.contains("waiting") {
            Ok(ProgramStatus::Waiting)
        } else if value.contains("stopped") {
            Ok(ProgramStatus::Stopped)
        } else if value.contains("paging") {
            Ok(ProgramStatus::Paging)
        } else {
            Err(CommandError::UnknownStatus)
        }
    }
}

impl TryFrom<(&str, Option<u32>)> for CommandUser {
    type Error = CommandError;
    fn try_from(value: (&str, Option<u32>)) -> Result<Self, Self::Error> {
        match value.0 {
            "list" => Ok(CommandUser::List),

            "exit" => Ok(CommandUser::Exit),
            _ => {
                let idx = match value.1 {
                    Some(x) => x,
                    None => return Err(CommandError::MissingParams),
                };
                match value.0 {
                    "kill" => Ok(CommandUser::Kill(idx)),
                    "restart" => Ok(CommandUser::Restart(idx)),
                    "launch" => Ok(CommandUser::Launch(idx)),
                    "status" => Ok(CommandUser::Status(idx)),
                    _ => Err(CommandError::UnknownCommand),
                }
            }
        }
    }
}
impl CommandUser {
    pub fn program_status(program: &TMProgram) -> Result<ProgramStatus, CommandError> {
        let data = match &program.child {
            None => return Err(CommandError::ProgramNotLaunched),
            Some(x) => format!("/proc/{}/status", x.id()),
        };
        let mut file = match File::open(data) {
            Ok(x) => x,
            Err(_) => return Err(CommandError::FailedOpenFile),
        };

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("cant read status file to end");
        let status_line = content.lines().nth(2);
        let status_line = match status_line {
            None => return Err(CommandError::RuntimeError),
            Some(x) => x,
        };
        Ok(status_line.try_into()?)
    }

    fn display_status(program: &mut TMProgram) -> Result<(), CommandError> {
        match program.child.as_mut() {
            None => return Err(CommandError::ProgramNotLaunched),
            Some(child) => {
                print!("{} : {} => ", program.config.command, child.id());
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("exited: {}", status.code().unwrap());
                        program.child = None;
                    }
                    Ok(None) => println!("{:?}", Self::program_status(program)?),
                    Err(_) => return Err(CommandError::RuntimeError),
                }
            }
        };
        Ok(())
    }
    fn list_childs(programs: &mut [TMProgram]) -> Result<(), CommandError> {
        println!("{} program running under out control", programs.len());
        for program in programs.iter_mut() {
            match CommandUser::display_status(program) {
                Err(e) => eprintln!(
                    "fetching status for [{}] raised error{e:?}",
                    program.config.command
                ),
                _ => {}
            }
        }
        Ok(())
    }

    fn status_child(programs: &mut [TMProgram], idx: u32) -> Result<(), CommandError> {
        match programs.get_mut(idx as usize) {
            None => Err(CommandError::WrongIndex),
            Some(x) => Self::display_status(x),
        }
    }

    fn kill_child(programs: &mut [TMProgram], idx: u32) -> Result<(), CommandError> {
        match programs.get_mut(idx as usize) {
            None => return Err(CommandError::WrongIndex),
            Some(x) => {
                match &mut x.child {
                    None => return Err(CommandError::ProgramNotLaunched),
                    Some(y) => {
                        y.kill().unwrap();
                        println!("program killed!");
                    }
                }
                x.child = None;
            }
        }
        Ok(())
    }

    fn launch_child(programs: &mut [TMProgram], idx: u32) -> Result<(), CommandError> {
        match programs.get_mut(idx as usize) {
            None => return Err(CommandError::WrongIndex),
            Some(program) => match program.child {
                Some(_) => eprintln!("program already launched"),
                None => {
                    program.launch();
                    if program.child.is_none() {
                        eprintln!("failed to launch program")
                    }
                }
            },
        };
        Ok(())
    }

    fn restart_child(programs: &mut [TMProgram], idx: u32) -> Result<(), CommandError> {
        Self::kill_child(programs, idx)?;
        Self::launch_child(programs, idx)?;
        Ok(())
    }

    pub fn exec(
        &self,
        programs: &mut [TMProgram],
        running: Arc<AtomicBool>,
    ) -> Result<(), CommandError> {
        match self {
            Self::Exit => {
                running.store(false, Ordering::SeqCst);
                Ok(())
            }
            Self::List => Self::list_childs(programs),
            Self::Status(x) => Self::status_child(programs, *x),
            Self::Kill(x) => Self::kill_child(programs, *x),
            Self::Launch(x) => Self::launch_child(programs, *x),
            Self::Restart(x) => Self::restart_child(programs, *x),
        }
    }
}
