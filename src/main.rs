use std::{
    error::Error,
    io::{stdin, stdout, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}, Mutex,
    },
};

use command::CommandError;

use crate::{command::CommandUser, config::TMConfig};
use crate::program::TMProgram;

pub mod command;
pub mod config;
mod program;
mod program_state;
mod program_status;

fn manage_programs(programs_arc: Arc<Mutex<Vec<TMProgram>>>) {
    let mut programs = programs_arc.lock().unwrap();
    for program in programs.iter_mut() {
        let _status_code = program.status();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let running_arc = Arc::new(AtomicBool::new(true));
    let programs_arc = Arc::new(Mutex::new(Vec::new()));
    std::panic::set_hook(Box::new(|info| {
        println!(
            "Panic occurred:\n\tlocation: {:?}\n\tpayload: {:?}",
            info.location(),
            info.payload()
        );
    }));

    let config = toml::from_str::<TMConfig>(&std::fs::read_to_string("config.toml")?)?;
    programs_arc
        .lock()
        .unwrap()
        .append(&mut config.launch_all()?);
    let programs = programs_arc.clone();
    let programs_manage = programs_arc.clone();
    let running = running_arc.clone();
    std::thread::spawn(move || {
        manage_programs(programs_manage);
    });
    while running.load(Ordering::SeqCst) {
        print!("$>");
        stdout().flush()?;
        let mut user_input = String::new();
        stdin().read_line(&mut user_input)?;
        let mut user_input = user_input.split(' ');
        let cmd = match user_input.next() {
            Some(x) => x.trim(),
            None => continue,
        };
        let val = user_input
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
