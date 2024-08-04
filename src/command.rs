use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::program_status::ProgramStatus;
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
    WrongIndex,
    UnknownCommand,
    MissingParams,
    RuntimeError(Box<dyn Error>),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CommandError {}

impl TryFrom<(&str, Option<u32>)> for CommandUser {
    type Error = CommandError;
    fn try_from(value: (&str, Option<u32>)) -> Result<Self, Self::Error> {
        match value.0 {
            "list" => Ok(CommandUser::List),
            "exit" => Ok(CommandUser::Exit),
            _ => {
                if value.0 != "kill"
                    && value.0 != "restart"
                    && value.0 != "launch"
                    && value.0 != "status"
                {
                    return Err(CommandError::UnknownCommand);
                }
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
    fn display_status(program: &mut TMProgram) -> Result<(), CommandError> {
        match program.status() {
            Err(e) => return Err(CommandError::RuntimeError(Box::new(e))),
            Ok(x) => {
                print!("{} => ", &program.config.command);
                match x {
                    ProgramStatus::Signal(signal) => println!("exited with code: {}", signal),
                    ProgramStatus::Code(code) => println!("exited with code: {}", code),
                    ProgramStatus::Running(state) => println!("{:?}", state),
                    ProgramStatus::Nothing => return Err(CommandError::ProgramNotLaunched),
                };
            }
        };
        Ok(())
    }
    fn list_childs(programs: &mut [TMProgram]) -> Result<(), CommandError> {
        println!("{} program running under out control", programs.len());
        for program in programs.iter_mut() {
            if let Err(e) = CommandUser::display_status(program) {
                eprintln!(
                    "fetching status for [{}] raised error{e:?}",
                    program.config.command
                )
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
                    Some(y) => y.kill().unwrap(),
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
                    if let Err(e) = program.launch() {
                        eprintln!("failed to launch program");
                        return Err(CommandError::RuntimeError(Box::new(e)));
                    }
                }
            },
        };
        Ok(())
    }

    fn restart_child(programs: &mut [TMProgram], idx: u32) -> Result<(), CommandError> {
        if let Err(x) = Self::kill_child(programs, idx) {
            match x {
                CommandError::ProgramNotLaunched => {}
                _ => return Err(x),
            }
        }
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
