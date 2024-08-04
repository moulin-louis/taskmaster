use std::{
    error::Error,
    ffi::c_int,
    io::{stdin, stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, Mutex,
    },
};

use command::CommandError;

use crate::program::TMProgram;
use crate::{command::CommandUser, config::TMConfig};

pub mod command;
pub mod config;
mod program;
mod program_state;
mod program_status;

static CONFIG: LazyLock<Mutex<TMConfig>> = LazyLock::new(|| {
    let content = match std::fs::read_to_string("config.toml") {
        Ok(x) => x,
        Err(e) => panic!("failed to read config toml: {}", e),
    };
    let config = match toml::from_str::<TMConfig>(&content) {
        Ok(x) => x,
        Err(e) => panic!("failed to parse content to toml: {}", e),
    };
    Mutex::new(config)
});

extern "C" {
    fn signal(signum: c_int, handler: extern "C" fn(c_int)) -> extern "C" fn(c_int);
}

const SIGHUP: c_int = 1;

extern "C" fn handle_sighup(_sig: c_int) {
    let programs = &mut CONFIG.lock().unwrap().programs;
    let content = match std::fs::read_to_string("config.toml") {
        Ok(x) => x,
        Err(e) => {
            eprintln!("failed to read config toml: {}", e);
            return;
        }
    };
    let new_config = match toml::from_str::<TMConfig>(&content) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("failed to parse content to toml: {}", e);
            return;
        }
    };

    programs.iter_mut().for_each(|it| {
        let new_program_config = match new_config.programs.get(it.0) {
            Some(x) => x.clone(),
            None => todo!("launch new program after config reload"),
        };
        *it.1 = new_program_config;
    });
}

fn manage_programs(programs_arc: Arc<Mutex<Vec<TMProgram>>>) {
    let mut programs = programs_arc.lock().unwrap();
    for program in programs.iter_mut() {
        let _status_code = program.status();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        //setup sighup config reload
        signal(SIGHUP, handle_sighup);
    }
    let running_arc = Arc::new(AtomicBool::new(true));
    let programs_arc = Arc::new(Mutex::new(Vec::new()));
    programs_arc
        .lock()
        .unwrap()
        .append(&mut CONFIG.lock()?.launch_all()?);
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
