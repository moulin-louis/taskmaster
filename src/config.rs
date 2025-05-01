use std::collections::HashMap;
use std::error::Error;

use libc::c_int;
use serde::Deserialize;

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
#[serde(rename_all = "lowercase")]
pub enum AutoRestart {
    Always,
    Never,
    UnExpected,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TMProgramConfig {
    /// The command to launch the program, must be findabled in $PATH or you need to set the full
    /// path of the command
    pub command: String,
    /// Argument to run the command with
    pub args: Vec<String>,
    pub process: u32,
    /// Whether to start the program when taskmaster launches
    /// Default: true
    pub autostart: bool,
    /// When to restart the program:
    ///  - always: Always restart the program, even on successful exits.
    ///  - never: Never restart the program.
    ///  - Unexpected: restart if unexpected exit status
    pub autorestart: AutoRestart,
    /// Expected exit status
    #[serde(default)]
    pub exit_status: Vec<c_int>,
    /// How many times a restart should be attempted before aborting
    pub number_restart: u32,
    /// How long the program should be running to be considered "successfully started" in secs.
    pub health_time: u32,
    /// Signal for graceful stop
    /// Default: SIGTERM
    pub stopsignal: String,
    /// Environment variables set before launching the program
    /// Default: Taskmaster environment
    ///TODO
    /// Working directory to set before launching the program
    /// Default: Taskmaster CWD
    pub cwd: Option<String>,
    /// umask to set before launching the program
    /// Default: 022
    pub umask: Option<i32>,
    ///Redirect stdout (optional)
    /// Default: Piped to taskmaster
    #[serde(default)]
    pub stdout: Option<String>,
    ///Redirect stderr (optional)
    /// Default: Piped to taskmaster
    #[serde(default)]
    pub stderr: Option<String>,
}
