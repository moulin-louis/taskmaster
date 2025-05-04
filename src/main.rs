use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, Mutex,
    },
};
use tokio::signal::unix::signal;
use tokio::{
    io::{self, stdin, AsyncBufReadExt, AsyncWriteExt},
    signal::unix::SignalKind,
};

use crate::program::TMProgram;
use crate::{command::CommandUser, config::TMConfig};

mod command;
mod config;
mod program;
mod program_state;
mod program_status;
mod shell;

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

async fn handle_sighup() {
    let mut stream = signal(SignalKind::hangup()).expect("Failed to create stream for SIGHUP");
    loop {
        stream.recv().await;
        println!("SIGHUP received");
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(handle_sighup());

    let running_arc = Arc::new(AtomicBool::new(true));
    let programs_arc = Arc::new(Mutex::new(Vec::new()));
    programs_arc
        .lock()
        .unwrap()
        .append(&mut CONFIG.lock()?.launch_all()?);
    let programs = programs_arc.clone();
    let running = running_arc.clone();

    let mut stdout = tokio::io::stdout();

    while running.load(Ordering::SeqCst) {
        stdout.write_all(b"$>").await?;
        stdout.flush().await?;
        let mut user_input = String::new();

        let mut reader = io::BufReader::new(stdin());

        reader.read_line(&mut user_input).await?;
        let mut user_input = user_input.split_whitespace();
        let cmd = match user_input.next() {
            Some(x) => x,
            None => continue,
        };
        //convert 2nd argument to u32
        let val = match user_input.next() {
            None => None,
            Some(x) => match x.parse() {
                Err(e) => {
                    eprintln!("Failed to parse argument to u32: {e}");
                    continue;
                }
                Ok(x) => Some(x),
            },
        };
        match CommandUser::try_from((cmd, val)) {
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
