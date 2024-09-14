use std::collections::HashMap;
use std::error::Error;

use serde_derive::Deserialize;

use crate::program::TMProgram;

#[derive(Deserialize, Debug, Clone)]
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
            let mut prog = TMProgram {
                config: config.clone(),
                child: None,
            };
            if !prog.config.autostart {
                continue;
            }
            if let Err(e) = prog.launch() {
                eprintln!("failed launching: {:?}", prog);
                return Err(Box::new(e));
            }
            res.push(prog);
        }
        Ok(res)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TMGlobalConfig {
    ///path were the log will be written
    pub logfile: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AutoRestart {
    Always,
    Never,
    UnExpected,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TMProgramConfig {
    /// The full command to start the program including the arguments if needed
    pub command: String,
    pub args: Vec<String>,
    /// Whether to start the program when taskmaster launches
    /// Default: true
    pub autostart: bool,
    /// When to restart the program:
    ///  - always: Always restart the program, even on successful exits.
    ///  - never: Never restart the program.
    ///  - Unexpected: restart if unexpected exit status
    /// Default: never
    pub autorestart: Option<AutoRestart>,
    /// Expected exit status
    #[serde(default)]
    pub exit_status: Option<Vec<i32>>,
    /// How long the program should be running to be considered "successfully started" in secs.
    /// Default: 1 secs
    pub time_success: Option<u32>,
    /// How many times a restart should be attempted before aborting
    /// Default: 1
    pub nbr_restart: Option<u32>,
    /// Signal for graceful stop
    /// Default: SIGTERM
    pub stopsignal: String,
    /// How long to wait after a graceful stop before killing the program in secs
    /// Default: 1 secs
    pub time_kill: u32,
    /// Environment variables set before launching the program
    /// Default: Taskmaster environment
    pub envs: Option<Vec<String>>,
    /// Working directory to set before launching the program
    /// Default: Taskmaster CWD
    pub cwd: Option<String>,
    /// umask to set before launching the program
    /// Default: no f*** idea
    pub umask: Option<String>,
    ///Redirect stdout (optional) path to file or "discarded"
    /// Default: Piped to taskmaster
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    ///Redirect stderr (optional): path to file or "discarded"
    /// Default: Piped to taskmaster
    pub stderr: Option<String>,
}
