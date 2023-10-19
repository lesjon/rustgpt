use std::fs;
use std::error::Error;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::openai::{ChatHistory, Messages};

pub const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
pub const DEFAULT_FILE_NAME: &str = "rustgpt/conversation.json";
pub const DEFAULT_CONFIG_FILE: &str = "rustgpt/config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    model: String,
    history_file: String,
    config_file: String,
}

impl Settings {
    pub fn clear_history(&self) -> Result<(), Box<dyn Error>> {
        let path = Path::new(&self.history_file);
        if !path.exists() {
            return Ok(());
        }
        let empty = Messages::new();
        let text = serde_json::to_string(&empty)?;
        Ok(fs::write(path, text)?)
    }
}

impl Settings {
    pub fn from_file(path: &str) -> Result<Settings, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&config_content)?)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_str = serde_json::to_string(self)?;
        let path = Path::new(&self.config_file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(fs::write(path, config_str)?)
    }

    pub fn create(history_file: &str, settings_file: &str, model: &str) -> Settings {
        let settings = Settings {
            model: model.to_string(),
            history_file: history_file.to_string(),
            config_file: settings_file.to_string(),
        };
        if let Err(e) = settings.save() {
            log::warn!("Could not save settings: {}", e);
        }
        settings
    }

    pub fn default() -> Settings {
        Self::create(DEFAULT_FILE_NAME, DEFAULT_CONFIG_FILE, DEFAULT_MODEL)
    }

    // read the history file if it exists, or create a new one if it doesn't
    // returns error if the file exists but could not be parsed
    pub fn get_history(&self) -> Result<Messages, Box<dyn Error>> {
        let path = Path::new(&self.history_file);
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            Ok(serde_json::from_str::<Messages>(&contents)?)
        } else {
            eprintln!("Creating new history");
            Ok(Messages(vec![]))
        }
    }

    pub fn write_history(&self, history: Messages) -> Result<(), Box<dyn Error>> {
        let path = Path::new(&self.history_file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let ser = serde_json::to_string(&history)?;
        Ok(fs::write(path, ser)?)
    }
}
