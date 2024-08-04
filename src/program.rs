use std::fs::File;
use std::io;
use std::process::{Child, Command, Stdio};

use crate::config::TMProgramConfig;

#[derive(Debug)]
pub struct TMProgram {
    pub config: TMProgramConfig,
    pub child: Option<Child>,
}

impl TMProgram {
    pub fn launch(&mut self) -> io::Result<()> {
        match Command::new(&self.config.command)
            .args(&self.config.args)
            .stdout(match &self.config.stdout {
                None => Stdio::piped(),
                Some(x) => Stdio::from(File::open(x)?),
            })
            .spawn()
        {
            Ok(x) => {
                self.child = Some(x);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
