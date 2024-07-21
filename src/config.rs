use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
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
    pub fn launch_all(&self) -> Result<Vec<TMProgram>, Box<dyn Error>> {
        let mut res: Vec<TMProgram> = Vec::new();
        for key in self.programs.keys() {
            let config = self.programs.get(key).unwrap();
            let stdout = match &config.stdout {
                None => Stdio::piped(),
                Some(x) => Stdio::from(File::open(x)?),
            };
            res.push(TMProgram {
                config: config.clone(),
                child: if config.autostart {
                    Some(
                        Command::new(&config.command)
                            .args(&config.args)
                            .stdout(stdout)
                            .spawn()?,
                    )
                } else {
                    None
                },
            })
        }
        Ok(res)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TMGlobalConfig {
    ///path were the log will be written
    pub logfile: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct TMProgramConfig {
    /// The full command to start the program including the arguments if needed
    pub command: String,
    pub args: Vec<String>,
    ///Whether to start the program when taskmaster launches
    pub autostart: bool,
    /// always: Always restart the program, even on successful exits.\n
    ///   never: Never restart the program.
    pub autorestart: String,
    ///Signal for graceful stop
    pub stopsignal: String,
    ///Redirect stdout (optional)
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    ///Redirect stderr (optional)
    pub stderr: Option<String>,
}
