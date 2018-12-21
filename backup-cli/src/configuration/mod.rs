use std::path::PathBuf;

use serde_derive::Deserialize;

mod yaml_reader;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Configuration {
    pub backup_paths: Vec<PathBuf>,
}

pub trait ConfigReader {
    type Error;
    fn read_config(&mut self) -> Result<Configuration, Self::Error>;
}