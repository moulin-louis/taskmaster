use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::Ordering;
use crate::program::ProgramStatus;
use crate::TMProgram;

#[derive(Debug)]
pub enum CommandUser {
    List,
    Kill(u32),
    Restart(u32),
    Launch(u32),
    Status(u32),
    Exit,
    Unknown,
}

#[derive(Debug)]
pub enum CommandError {
    ProgramNotLaunched,
    FailedOpenFile,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CommandError {}

impl From<&str> for ProgramStatus {
    fn from(value: &str) -> Self {
        if value.contains("sleeping") {
            ProgramStatus::Sleeping
        } else if value.contains("running") {
            ProgramStatus::Running
        } else if value.contains("waiting") {
            ProgramStatus::Waiting
        } else if value.contains("stopped") {
            ProgramStatus::Stopped
        } else if value.contains("paging") {
            ProgramStatus::Paging
        } else {
            ProgramStatus::Unknown
        }
    }
}

impl From<(&str, Option<u32>)> for CommandUser {
    fn from(value: (&str, Option<u32>)) -> Self {
        match value.0 {
            "list" => CommandUser::List,
            "kill" => CommandUser::Kill(value.1.expect("no id for kill")),
            "restart" => CommandUser::Restart(value.1.expect("no id for restart")),
            "launch" => CommandUser::Launch(value.1.expect("no id for launch")),
            "status" => CommandUser::Status(value.1.expect("no id for status")),
            "exit" => CommandUser::Exit,
            _ => CommandUser::Unknown,
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
            Err(e) => {
                eprintln!("got error: {e} opening proc file");
                return Err(CommandError::FailedOpenFile);
            }
        };

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("cant read status file to end");
        let status_line = content.lines().nth(2).unwrap();
        Ok(ProgramStatus::from(status_line))
    }

    fn find_child(programs: &mut [TMProgram], val: u32) -> Option<&mut TMProgram> {
        programs
            .iter_mut()
            .find(|it| it.is_launched() && it.child.as_ref().unwrap().id() == val)
    }

    fn display_status(program: &mut TMProgram) {
        match Self::program_status(program) {
            Err(e) => eprintln!("failed to fetch status: {}", e),
            Ok(status) => match program.child.as_mut() {
                None => eprintln!("program ({}) isn't launched", program.config.command),
                Some(child) => {
                    print!("{} : {} => ", program.config.command, child.id());
                    match child.try_wait() {
                        Ok(Some(status)) => println!("exited: {}", status.code().unwrap()list),
                        Ok(None) => println!("{:?}", status),
                        Err(_) => eprintln!("failed to get status"),
                    }
                }
            },
        };
    }
    fn list_childs(programs: &mut [TMProgram]) {
        println!("{} program running under out control", programs.len());
        for program in programs.iter_mut() {
            CommandUser::display_status(program);
        }
    }

    fn status_child(programs: &mut [TMProgram], val: u32) {
        match CommandUser::find_child(programs, val) {
            None => eprintln!("this id isnt under out control"),
            Some(x) => Self::display_status(x),
        }
    }

    fn kill_child(programs: &mut [TMProgram], val: u32) {
        match CommandUser::find_child(programs, val) {
            None => eprintln!("this id isn't under our control"),
            Some(x) => {
                match &mut x.child {
                    None => {
                        eprintln!("this id isn't launched");
                        return;
                    }
                    Some(y) => {
                        y.kill().unwrap();
                        println!("program killed!");
                    }
                }
                x.child = None;
            }
        }
    }

    pub fn exec(&self, programs: &mut [TMProgram], running: Arc<AtomicBool>) {
        match self {
            Self::Exit => running.store(false, Ordering::SeqCst),
            Self::List => Self::list_childs(programs),
            Self::Status(x) => Self::status_child(programs, *x),
            Self::Kill(x) => Self::kill_child(programs, *x),
            // Self::Launch(x) => Self::launch_child(programs, *x),
            _ => println!("unhandled command"),
        }
    }
}
