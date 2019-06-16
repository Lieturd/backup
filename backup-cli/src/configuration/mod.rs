use std::fs::File;
use std::path::PathBuf;
use std::vec::Vec;

use serde_derive::Deserialize;

mod yaml_reader;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Configuration {
    pub backup_paths: Vec<PathBuf>,
    pub server_host: String,
    pub server_port: u16,
}

pub trait ConfigReader {
    type Error;
    fn read_config(&mut self) -> Result<Configuration, Self::Error>;
}

pub fn read_config(path: &str) -> Result<Configuration, Box<std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = yaml_reader::YamlReader::new(file);
    return reader.read_config().map_err(|e| e.into() );
}
