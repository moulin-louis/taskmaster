use crate::Ordering;
use crate::TMProgram;
use crate::RUNNING;
use std::error::Error;
use std::fs::File;
use std::io::Read;

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

pub enum CommandStatus {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Paging,
    Unknown,
}

impl From<&str> for CommandStatus {
    fn from(value: &str) -> Self {
        if (value.contains("sleeping")) {
            CommandStatus::Sleeping
        } else if (value.contains("running")) {
            CommandStatus::Running
        } else if (value.contains("waiting")) {
            CommandStatus::Waiting
        } else if value.contains("stopped") {
            CommandStatus::Stopped
        } else if value.contains("paging") {
            CommandStatus::Paging
        } else {
            CommandStatus::Unknown
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
    fn child_status(id: u32) -> Result<CommandStatus, Box<dyn Error>> {
        let data = format!("/proc/{id}/status");
        let mut file = match File::open(data) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("failed to open process status file, most likely perm error or process isnt running");
                return Err(Box::new(e));
            }
        };

        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("cant read status file to end");
        let status_line = content
            .lines()
            .nth(2)
            .expect("nothing on line 3 for status file");

        Ok(CommandStatus::from(status_line))
    }

    fn list_childs(programs: &Vec<TMProgram>) {
        for program in programs {
            let child = &program.child;
            let config = &program.config;
            let status = Self::child_status(child.id());
        }
    }

    pub fn exec(&self, programs: &Vec<TMProgram>) {
        match self {
            Self::Exit => RUNNING.store(false, Ordering::SeqCst),
            Self::List => Self::list_childs(programs),
            _ => println!("unhandled command"),
        }
    }
}