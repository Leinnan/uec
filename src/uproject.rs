#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    pub Name: String,
    Type: String,
    LoadingPhase: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plugin {
    Name: String,
    Enabled: bool,
    TargetAllowList: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    FileVersion: i32,
    EngineAssociation: String,
    Category: String,
    Description: String,
    Modules: Vec<Module>,
    Plugins: Vec<Plugin>,
}

pub fn read_config(path: &str) -> std::io::Result<Config> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let config: Config = serde_json::from_str(&data)?;
    Ok(config)
}
impl Config {
    pub fn find_editor_module(&self) -> Option<&Module> {
        self.Modules.iter().find(|m| m.Type == "Editor")
    }
}
