use std::fs;
use std::path;

use crate::derfile::*;
use crate::error::*;
use crate::utils::debug;

#[derive(Clone, Debug)]
pub struct Config {
    derfile: Derfile,
}

impl Default for Config {
    fn default() -> Self {
        let mut derfile = Derfile::default();
        derfile.templates.insert("default".to_string(), Template::default());
        Self {
            derfile
        }
    }
}

impl Config {
    pub fn load<P: AsRef<path::Path>>(path: &P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            if debug() {
                println!(
                    "[INFO] Config file {} not found, using default.",
                    path.to_string_lossy()
                )
            }
            return Ok(Self::default());
        } else {
            let buffer = fs::read_to_string(path)?;
            let derfile = Derfile::load_derfile(buffer, path)?;
            if debug() {
                println!("[INFO] Loaded config file: {}", path.to_string_lossy())
            }
            return Ok(Self { derfile });
        }
    }

    pub fn _write<P: AsRef<path::Path>>(&self, path: &P) -> Result {
        let path = path.as_ref();

        let string = self.derfile.to_string();
        fs::write(path, string)?;
        if debug() {
            println!(
                "[INFO] Config file was successfully written to {}",
                path.to_string_lossy()
            );
        }
        Ok(())
    }

    pub fn merge_vars(&self) -> Vec<(&String, &Variable)> {
        self.derfile
            .vars
            .iter()
            .collect::<Vec<(&String, &Variable)>>()
            .to_vec()
    }

    pub fn config_template(&mut self) -> &mut Template {
        let template = self.derfile.get_template(&"default".to_string()).unwrap();
        return template
    }
}
