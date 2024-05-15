use std::{
    fs::File,
    io::{self, stdout, Read, Write},
    path::PathBuf,
};

use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Represents the applications Configuration
#[derive(Serialize, Deserialize)]
pub struct Configuration {
    /// The E-Mail address to use for logging in to Factorial
    pub email: String,
}
impl Configuration {
    /// Generates a default configuration
    fn default() -> Configuration {
        Configuration {
            email: String::from(""),
        }
    }

    /// Retrieves the path of the applications configuration file. If the file does not exist, a
    /// configuration file with default values is generated. Missing parent directories will also
    /// be created.
    ///
    /// # Errors
    /// - Returns an error if home directory could not be retrieved from the OS.
    /// - Returns an error if missing parent directories could not be created.
    /// - Returns an error if the configuration file could not be created.
    fn get_config_file_path() -> anyhow::Result<PathBuf> {
        let config_dir = match ProjectDirs::from("", "", "Tracktorial") {
            Some(config_dir) => config_dir.config_local_dir().to_owned(),
            None => return Err(anyhow!("Could not determine the home directory.")),
        };
        std::fs::create_dir_all(&config_dir)?;
        let config_file_path = config_dir.join("config.json");
        match config_file_path.try_exists() {
            Ok(true) => Ok(config_file_path),
            Ok(false) => {
                File::create(&config_file_path)?.write_all(
                    serde_json::to_string_pretty(&Configuration::default())
                        .unwrap()
                        .as_bytes(),
                )?;
                Ok(config_file_path)
            }
            Err(_) => Err(anyhow!("test")),
        }
    }
    /// Reads the configuration from the configuration file. If the file does not exist, it will be
    /// created as well as potentially missing parent directories.
    ///
    /// # Errors
    /// - Returns an error if the configuration file or its parent directories could not be retrieved or created.
    /// - Returns an error if the configuration files contents are invalid.
    pub fn get_config() -> anyhow::Result<Configuration> {
        let mut config_file = File::open(Self::get_config_file_path()?)?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;
        let config: Configuration = serde_json::from_str(content.as_str())?;
        Ok(config)
    }

    /// Writes the current configuration to the configuration file.
    ///
    /// # Errors
    /// - Returns an error if the configuration file or its parent directories could not be retrieved or created.
    pub fn write_config(&self) -> anyhow::Result<()> {
        File::create(Self::get_config_file_path()?)?
            .write_all(serde_json::to_string_pretty(self).unwrap().as_bytes())?;
        Ok(())
    }

    pub fn prompt_for_email(&mut self) -> anyhow::Result<()> {
        let mut buffer = String::new();
        println!("Enter E-Mail address: ");
        io::stdin().read_line(&mut buffer)?;
        self.email = String::from(&buffer.trim().to_owned());
        self.write_config()?;
        Ok(())
    }
}
