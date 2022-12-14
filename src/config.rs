use std::env;
use std::fmt;
use std::fs;
use std::path;

use crate::derfile::{Template, Variable};
use crate::derfile::{CODE_KEYWORDS, CODE_SEP, VAR_PREF};
use crate::error::*;
use crate::utils::{debug, execute_code};

type Variables = Vec<Variable>;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub(crate) template: Template,
    pub(crate) vars: Variables,
}

impl Config {
    pub fn load<P: AsRef<path::Path>>(path: &P) -> Result<Self> {
        let path = path.as_ref();
        if let Ok(read_file) = fs::read_to_string(path) {
            Config::parse(&read_file)
        } else {
            return Err(format!("Unable to read config file: {}", path.to_string_lossy()).into());
        }
    }

    pub fn parse<S: AsRef<str>>(input: &S) -> Result<Self> {
        let lines = input.as_ref().lines();
        let mut config = Config::default();

        for line in lines {
            // line: "hello = something"
            if line.contains('=') {
                let split = line.split_at(line.find('=').unwrap());
                // "hello"
                let left_part = split.0.trim();
                // "something"
                let right_part = split.1.trim().strip_prefix('=').unwrap().trim();

                if left_part.contains(VAR_PREF) {
                    let mut env_keyword_string = <&str>::clone(&CODE_KEYWORDS[0]).to_string();
                    env_keyword_string.push_str(CODE_SEP);
                    let var_name = left_part.strip_prefix(VAR_PREF).unwrap();

                    if right_part.starts_with(CODE_SEP) && right_part.ends_with(CODE_SEP) {
                        let code = right_part.strip_prefix(CODE_SEP).unwrap();
                        let code = code.strip_suffix(CODE_SEP).unwrap();
                        if let Ok(code_result) = execute_code(code) {
                            config
                                .vars
                                .push(Variable::new(var_name.to_string(), vec![code_result]))
                        } else if debug() {
                            println!("[\x1b[31mERROR\x1b[0m] Unable to execute code: {code}");
                        }
                    } else if right_part.starts_with(&env_keyword_string)
                        && right_part.ends_with(CODE_SEP)
                    {
                        let env = right_part.strip_prefix(&env_keyword_string).unwrap();
                        let env = env.strip_suffix(CODE_SEP).unwrap();
                        if let Ok(var) = env::var(env) {
                            config
                                .vars
                                .push(Variable::new(var_name.to_string(), vec![var]))
                        } else if debug() {
                            println!(
                                "[\x1b[31mERROR\x1b[0m] Unable to read value of environmental variable: ${env}",
                            );
                        }
                    } else {
                        config
                            .vars
                            .push(Variable::new(var_name.to_string(), vec![right_part.into()]))
                    }
                } else {
                    match left_part {
                        "extensions" => {
                            if right_part.contains(',') {
                                for each in right_part.split(',') {
                                    config.template.add_extension(each.to_string())
                                }
                            } else {
                                config.template.add_extension(right_part.to_string())
                            }
                        }
                        "hostnames" => {
                            if right_part.contains(',') {
                                for each in right_part.split(',') {
                                    config.template.add_hostname(each.to_string())
                                }
                            } else {
                                config.template.add_hostname(right_part.to_string())
                            }
                        }
                        "recursive" => {
                            if right_part == "true" {
                                config.template.set_recursive(true)
                            } else {
                                config.template.set_recursive(false)
                            }
                        }
                        "parse_files" => {
                            if right_part == "true" {
                                config.template.set_parse_files(true)
                            } else {
                                config.template.set_parse_files(false)
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn load_default() -> Result<Self> {
        let default_path = path::PathBuf::from(format!(
            "/home/{}/.config/der/config",
            execute_code("whoami")?
        ));
        let config = Config::default();

        if default_path.exists() {
            return Config::load(&default_path);
        } else {
            fs::create_dir_all(default_path.parent().unwrap())?;
            fs::write(&default_path, config.to_string())?;
            if debug() {
                println!(
                    "[\x1b[32mINFO\x1b[0m] Wrote default config file to: {}",
                    default_path.to_string_lossy()
                )
            }
        }

        Ok(config)
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "hostnames = {}", self.template.serialize_hostnames())?;
        writeln!(f, "extensions = {}", self.template.serialize_extensions())?;
        writeln!(f, "recursive = {}", self.template.recursive)?;
        writeln!(f, "parse_files = {}", self.template.parse_files)?;
        for var in &self.vars {
            writeln!(f, "${} = {}", var._name, var.serialize())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn config() {
        println!("{}", Config::load_default().unwrap())
    }
}
