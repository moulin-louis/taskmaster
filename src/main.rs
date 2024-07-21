use std::{
    error::Error,
    io::{stdin, stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use command::CommandError;

use crate::program::TMProgram;
use crate::{command::CommandUser, config::TMConfig};

pub mod command;
pub mod config;
mod program;

fn main() -> Result<(), Box<dyn Error>> {
    let running_arc: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let programs_arc: Arc<Mutex<Vec<TMProgram>>> = Arc::new(Mutex::new(Vec::new()));

    let config: TMConfig = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let programs = &mut config.launch_all()?;
    programs_arc.lock().unwrap().append(programs);
    let programs = programs_arc.clone();
    let running = running_arc.clone();
    while running.load(Ordering::SeqCst) {
        print!("$>");
        stdout().flush()?;
        let mut user_input: String = String::new();
        stdin().read_line(&mut user_input)?;
        let mut user_input = user_input.split(' ');
        let cmd = match user_input.next() {
            Some(x) => x.trim(),
            None => continue,
        };
        let val: Option<u32> = user_input
            .next()
            .filter(|x| !x.is_empty())
            .and_then(|x| x.trim().parse().ok());
        let cmd: Result<CommandUser, CommandError> = (cmd, val).try_into();
        match cmd {
            Ok(cmd) => {
                if let Err(e) = cmd.exec(&mut programs.lock().unwrap(), running.clone()) {
                    eprintln!("command {cmd:?} raised error {e:?}");
                }
            }
            Err(e) => eprintln!("parsing command raised {e:?}"),
        };
    }
    running.store(false, Ordering::SeqCst);
    programs.lock().unwrap().iter_mut().for_each(|x| {
        if let Some(child) = &mut x.child {
            if let Err(e) = &child.kill() {
                eprintln!("failed to kill {x:?}: {e}");
            }
        }
    });
    Ok(())
}
