use serde_derive::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::process::{Child, Command, Stdio};

#[derive(Deserialize, Debug)]
pub struct TMConfig {
    #[serde(rename = "global")]
    global: TMGlobalConfig,
    #[serde(rename = "programs")]
    programs: HashMap<String, TMProgramConfig>,
}

#[derive(Deserialize, Debug)]
pub struct TMGlobalConfig {
    logfile: String,
}

#[derive(Deserialize, Debug)]
pub struct TMProgramConfig {
    command: String,
    autostart: bool,
    autorestart: String,
    exitcodes: Vec<u8>,
    startsecs: u32,
    stopsignal: String,
    stopwaitsecs: u32,
    #[serde(default)]
    stdout: Option<String>,
    #[serde(default)]
    stderr: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "config.toml";

    let contents = std::fs::read_to_string(filename)?;
    let config: TMConfig = toml::from_str(&contents)?;
    let mut childs: Vec<Child> = Vec::new();
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
        childs.push(child);
    }

    Ok(())
}
