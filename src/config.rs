use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Represents the applications Configuration
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Configuration {
    /// The E-Mail address to use for logging in to Factorial
    #[serde(default = "default_mail")]
    pub email: String,
    /// The location you work from, either office or home
    #[serde(default = "default_location")]
    pub location_type: String,
    /// The Id Factorial uses to identify the user
    #[serde(default = "default_user_id")]
    pub user_id: String,
    /// The default amount of working hours per week
    #[serde(default = "default_hours")]
    pub working_hours: f32,

    #[serde(default = "default_working_days")]
    pub working_week_days: Vec<String>,
    /// The preffered amount of working hours every day, defaults to the working_hours divided by
    /// the amount of working week days
    #[serde(default = "default_duration")]
    pub shift_duration: f32,
    /// The maximum amount in minutes that the shift and break will be started before or after the
    /// specified time if the randomization option is enabled.
    #[serde(default = "default_rand_range")]
    pub max_rand_range: u16,
}
impl Configuration {
    /// Generates a default configuration
    pub fn default() -> Configuration {
        Configuration {
            email: default_mail(),
            location_type: default_location(),
            user_id: default_user_id(),
            working_hours: default_hours(),
            working_week_days: default_working_days(),
            shift_duration: default_duration(),
            max_rand_range: default_rand_range(),
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
        let content = fs::read_to_string(Self::get_config_file_path()?)?;
        let config: Configuration = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Writes the current configuration to the configuration file.
    ///
    /// # Errors
    /// - Returns an error if the configuration file or its parent directories could not be retrieved or created.
    pub fn write_config(&self) -> anyhow::Result<()> {
        File::create(Self::get_config_file_path()?)?
            .write_all(serde_json::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }

    /// Prompt the user for email address
    /// # Errors
    /// Return an error if the address could not be read from stdin or could not be written to the
    /// configuration file.
    pub fn prompt_for_email(&mut self) -> anyhow::Result<()> {
        let mut buffer = String::new();
        println!("Enter E-Mail address: ");
        std::io::stdin().read_line(&mut buffer)?;
        self.email = String::from(&buffer.trim().to_owned());
        self.write_config()?;
        Ok(())
    }
}

fn default_mail() -> String {
    "".to_string()
}
fn default_location() -> String {
    "office".to_string()
}
fn default_user_id() -> String {
    "".to_string()
}
fn default_hours() -> f32 {
    0.00
}
fn default_duration() -> f32 {
    0.0
}
fn default_working_days() -> Vec<String> {
    Vec::new()
}
fn default_rand_range() -> u16 {
    30
}
