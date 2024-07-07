use serde_derive::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TMConfig {
    #[serde(rename = "global")]
    pub global: TMGlobalConfig,
    #[serde(rename = "programs")]
    pub programs: HashMap<String, TMProgramConfig>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TMGlobalConfig {
    pub logfile: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
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
