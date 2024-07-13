use std::{
    error::Error,
    io::{stdin, stdout, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}, Mutex,
    },
};

use crate::{command::CommandUser, config::TMConfig};
use crate::program::TMProgram;

pub mod command;
pub mod config;
mod program;

// fn gb_programs(programs: &mut Vec<TMProgram>) -> Result<(), Box<dyn Error>> {
//     let mut killed_program: Vec<u32> = Vec::new();
//     for program in &mut *programs {
//         let status = match CommandUser::program_status(program) {
//             Ok(x) => x,
//             Err(e) if e.downcast_ref::<io::Error>().unwrap().kind() == ErrorKind::NotFound => {
//                 return Ok(());
//             }
//             Err(e) => return Err(e),
//         };
//         if let ProgramStatus::Unknown = status {
//             killed_program.push(program.child.as_ref().unwrap().id())
//         }
//     }
//     programs.retain(|program| !killed_program.contains(&program.child.as_ref().unwrap().id()));
//     Ok(())
// }

fn main() -> Result<(), Box<dyn Error>> {
    let running_arc: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let programs_arc: Arc<Mutex<Vec<TMProgram>>> = Arc::new(Mutex::new(Vec::new()));

    let config: TMConfig = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let programs = &mut config.launch_all()?;
    programs_arc.lock().unwrap().append(programs);
    // let cleanup_thread: JoinHandle<()> = thread::spawn({
    //     let programs = programs_arc.clone();
    //     let running = running_arc.clone();
    //     move || loop {
    //         if !running.load(Ordering::SeqCst) {
    //             break;
    //         }
    //         gb_programs(&mut programs.lock().unwrap()).unwrap();
    //         thread::sleep(std::time::Duration::from_millis(500)); // Check periodically
    //     }
    // });
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
        let cmd: CommandUser = (cmd, val).into();
        cmd.exec(&mut programs.lock().unwrap(), running.clone());
    }
    running.store(false, Ordering::SeqCst);
    // cleanup_thread.join().unwrap_or_else(|e| eprintln!("failed to join cleanup thread = {e:?}"));
    programs
        .lock()
        .unwrap()
        .iter_mut()
        .for_each(|x| match &mut x.child {
            None => {}
            Some(x) => x.kill().unwrap(),
        });
    Ok(())
}
