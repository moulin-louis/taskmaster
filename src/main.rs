use std::error::Error;
use std::io::{self};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

pub mod config;
use crate::config::{TMConfig, TMProgramConfig};

pub mod command;
use crate::command::CommandUser;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Debug)]
pub struct TMProgram {
    config: TMProgramConfig,
    child: Child,
}

// extern "C" {
//     fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> c_int;
// }

// extern "C" fn sig_handler(_sig: c_int) {
//     RUNNING.store(false, Ordering::SeqCst);
// }

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "config.toml";

    let contents = std::fs::read_to_string(filename)?;
    let config: TMConfig = toml::from_str(&contents)?;
    let mut programs: Vec<TMProgram> = Vec::new();
    for (_key, config) in config.programs {
        let program: Vec<&str> = config.command.split(' ').collect();
        let cmd = program[0];
        let args = &program[1..program.len()];
        let child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;
        println!("child id = {}", child.id());
        println!("bat /proc/{}/status", child.id());
        programs.push(TMProgram { config, child });
    }
    // unsafe {
    //     let res = signal(2, sig_handler);
    //     if res == -1 {
    //         let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
    //         println!("ernno = {errno}");
    //         std::process::exit(1);
    //     }
    //     println!("res signal = {res}");
    // }
    let stdin = io::stdin();
    while RUNNING.load(Ordering::SeqCst) {
        /*
        Check for child process completion (or other conditions)
        You might want to use something like a channel or a WaitGroup to m ca anage child termination gracefully
        */
        let mut user_input: String = String::new();
        stdin
            .read_line(&mut user_input)
            .expect("failed to read a line");
        let mut user_input = user_input.split(' ');
        let cmd = match user_input.next() {
            Some(x) => x.trim(),
            None => continue,
        };
        let val: Option<u32> = match user_input.next() {
            Some(x) => {
                if x.is_empty() {
                    None
                } else {
                    Some(x.trim().parse().unwrap())
                }
            }
            None => None,
        };
        let cmd: CommandUser = (cmd, val).into();
        println!("cmd = {cmd:?}");
        cmd.exec(&programs);
    }

    println!("kill all childs to avoir zombie process");
    for mut program in programs {
        let pid = program.child.id();
        program
            .child
            .kill()
            .expect(format!("Unable to kill child {program:?}").as_str());
        println!("process {pid} killed");
    }
    Ok(())
}
