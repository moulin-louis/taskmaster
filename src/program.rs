use std::process::Child;

use crate::config::TMProgramConfig;

#[derive(Debug)]
pub enum ProgramStatus {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Paging,
    Unknown,
}

impl Default for ProgramStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct TMProgram {
    pub config: TMProgramConfig,
    pub child: Option<Child>,
}

impl TMProgram {
    pub fn is_launched(&self) -> bool {
        self.child.is_some()
    }
    pub fn launch(&self) {
        
    }
}
