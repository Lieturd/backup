use std::io::Read;

use serde_yaml::Error as YamlError;

use crate::configuration::{Configuration, ConfigReader};

pub struct YamlReader<R> {
    inner: R,
}

impl<R> YamlReader<R> {
    pub fn new(inner: R) -> YamlReader<R> {
        YamlReader {
            inner: inner,
        }
    }
}

impl<R> ConfigReader for YamlReader<R>
    where R: Read,
{
    type Error = YamlError;

    fn read_config(&mut self) -> Result<Configuration, Self::Error> {
        serde_yaml::from_reader(&mut self.inner)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::YamlReader;
    use crate::configuration::{Configuration, ConfigReader};

    #[test]
    fn test_read_proper_config() {
        let static_config = Cursor::new(r#"
            backup_paths:
              - foo
              - bar
              - baz
        "#);
        let mut config_reader = YamlReader::new(static_config);

        let config_result = config_reader.read_config();
        let config_should_be = Configuration {
            backup_paths: vec!["foo".into(), "bar".into(), "baz".into()],
        };

        assert_eq!(config_result.unwrap(), config_should_be);
    }

    #[test]
    fn test_read_improper_config() {
        let static_config = Cursor::new(r#"
            backup_paaths:
              - foo
              - bar
              - baz
        "#);
        let mut config_reader = YamlReader::new(static_config);

        let config_result = config_reader.read_config();

        assert!(config_result.is_err());
    }
}