use std::collections::HashMap;
use std::error::Error;
use std::process::{Command, Stdio};
use serde_derive::Deserialize;

use crate::program::TMProgram;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TMConfig {
    #[serde(rename = "global")]
    pub global: TMGlobalConfig,
    #[serde(rename = "programs")]
    pub programs: HashMap<String, TMProgramConfig>,
}

impl TMConfig {
    pub fn launch_all(&self) -> Result<Vec<TMProgram>, Box<dyn Error>>  {
        let mut res: Vec<TMProgram> = Vec::new();
        for key in self.programs.keys() {
            let config = self.programs.get(key).unwrap();
            let program: Vec<&str> = config.command.split(' ').collect();
            let cmd = program[0];
            let args = &program[1..program.len()];
            let child = Command::new(cmd)
                .args(args)
                .stdout(Stdio::piped())
                .spawn()?;
            println!("child id = {}", child.id());
            res.push(TMProgram {config: config.clone(), child: Some(child) })
        }
        Ok(res)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TMGlobalConfig {
    pub logfile: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct TMProgramConfig {
    pub command: String,
    pub autostart: bool,
    pub autorestart: String,
    pub exitcodes: Vec<u8>,
    pub startsecs: u32,
    pub stopsignal: String,
    pub stopwaitsecs: u32,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
}
