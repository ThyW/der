use std::collections::HashMap;
use std::env;
use std::fmt;
use std::path;

use crate::config::*;
use crate::error::*;
use crate::utils::*;

/// Symbols for derfile parsing, these can be changed before compilation.
pub const TEMPLATE_LEFT: &str = "[";
pub const TEMPLATE_RIGHT: &str = "]";
pub const CODE_SEP: &str = "`";
pub const VAR_PREF: &str = "$";
pub const VAR_ADD: &str = ":"; // variable separator for adding values to a variable
pub const CODE_KEYWORDS: [&str; 1] = ["env"];

/// A template section of a derfile.
#[derive(Debug, Clone, Default)]
pub struct Template {
    /// Templates name, or in this case its path.
    pub name: String,
    /// Name of the output file.
    pub final_name: String,
    /// A list of hostnames, for which this template should be parsed.
    pub hostnames: Vec<String>,
    /// A path to the directory in which the parsed file should be placed.
    pub apply_path: String,
    /// [Dirctory only fields]
    /// Whether all template files found in this directory and its subdirectories should be parsed.
    pub parse_files: bool,
    /// Extensions to look for when searching a directory for template files.
    pub extensions: Vec<String>,
    /// If this flag is true, a recursive search the directory and all its child directories will
    /// be performed. If false, only the directory will be visited, all its subdirectories
    /// ignored.
    pub recursive: bool,
}

/// A single derfile variable.
#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub _name: String,
    /// Value[s] of the variable.
    pub value: Vec<String>,
}

/// Representation of a derfile.
#[derive(Debug, Clone, Default)]
pub struct Derfile {
    /// Key value pairs of tempale names(their paths) and templates.
    pub templates: HashMap<String, Template>,
    /// Key value pairs of variable names and their values.
    pub vars: HashMap<String, Variable>,
    /// Absolute path to derfile.
    pub(crate) path: path::PathBuf,
    /// Which fileds are empty
    pub(crate) empty_fields: u8,
}

/// Just some setters for working with templates.
impl Template {
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_final_name(&mut self, final_name: String) {
        self.final_name = final_name
    }

    pub fn add_hostname(&mut self, hostname: String) {
        self.hostnames.push(hostname)
    }

    pub fn set_apply_path(&mut self, apply_path: String) {
        self.apply_path = apply_path
    }

    pub fn set_recursive(&mut self, arg: bool) {
        self.recursive = arg
    }

    pub fn set_parse_files(&mut self, arg: bool) {
        self.parse_files = arg
    }

    pub fn add_extension(&mut self, ext: String) {
        self.extensions.push(ext)
    }

    pub(crate) fn serialize_hostnames(&self) -> String {
        self.hostnames.join(",")
    }

    pub(crate) fn serialize_extensions(&self) -> String {
        self.extensions.join(",")
    }
}

impl Variable {
    /// Construct a new variable.
    pub fn new(_name: String, value: Vec<String>) -> Self {
        Self { _name, value }
    }

    pub(crate) fn serialize(&self) -> String {
        self.value.join(", ")
    }
}

impl Derfile {
    /// Add a new empty template, give its name(path).
    pub fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    /// Get a mutable reference to a template if we have its name.
    pub fn get_template<S: AsRef<str>>(&mut self, name: &S) -> Option<&mut Template> {
        self.templates.get_mut(name.as_ref())
    }

    /// Create a new varialbe.
    pub fn add_var(&mut self, name: String, value: Vec<String>) {
        self.vars.insert(name.clone(), Variable::new(name, value));
    }

    /// Get a value of a varialbe.
    pub fn get_var<S: AsRef<str>>(&mut self, name: &S) -> Option<&mut Variable> {
        self.vars.get_mut(name.as_ref())
    }

    pub fn with_config(&mut self, config: &Config) {
        for each in &config.vars {
            self.vars.insert(each._name.clone(), each.clone());
        }

        self.add_template("[default-template]".to_string());
        if let Some(default_template) = self.get_template(&"[default-template]") {
            *default_template = config.template.clone();
        }
    }

    /// Parse a template file, which has already been loaded.
    fn parse(self) -> Self {
        // Create a new output derfile into which we will parse our current defile.
        let mut new_derfile: Derfile = Default::default();
        // We create a clone of self for simpler data manipulation.
        let mut self_clone = self.clone();
        let default_template: Template;
        if let Some(dt) = self_clone.get_template(&"[default-template]") {
            default_template = dt.clone();
        } else {
            default_template = Template::default();
        }

        for (template_name, template) in self.templates.iter() {
            // always skip config template
            if template_name == "[default-template]" {
                continue;
            }
            // For each template in our current derfile, we create a new temlate field in the new
            // derfile.
            new_derfile.add_template(template_name.clone());

            // Then we get a mutable reference to the new template so we can parse and isert data
            // into it.
            let mut new_template = new_derfile.get_template(&template_name).unwrap();
            new_template.name = template_name.to_string();

            // Then we check through all fields, parse them and add them to the new defile.
            if template.final_name.starts_with(VAR_PREF) {
                if template.final_name.contains(VAR_ADD) {
                    // First, we split by VAR_ADD. On one side we get var $varname and on the other
                    // side we get the value that we want to add.
                    let split_variable = template
                        .final_name
                        .split_at(template.final_name.find(VAR_ADD).unwrap());
                    let variable_name =
                        split_variable.0.strip_prefix(VAR_PREF).unwrap().to_string();
                    let additional_value = split_variable.1.strip_prefix(VAR_ADD).unwrap();

                    // Then after we retrieve the value of the variable, we can then add to its
                    // the additional value. We do this for all variables except the boolean ones.
                    if let Some(variable) = self_clone.get_var(&variable_name) {
                        let mut value = variable.value[0].clone();
                        value.push_str(additional_value);
                        new_template.final_name = value;
                    }
                } else {
                    let variable_name = template
                        .final_name
                        .strip_prefix(VAR_PREF)
                        .unwrap()
                        .to_string();

                    if let Some(variable) = self_clone.get_var(&variable_name) {
                        new_template.final_name = variable.value[0].clone(); // only take the fist value, since we only accept one final file name
                    }
                }
            } else {
                new_template.final_name = template.final_name.clone();
            }

            // This is probably the ugliest part of this function, since we have to do a lot of path manipulation
            // magic here.
            if template.apply_path.starts_with(VAR_PREF) {
                if template.apply_path.contains(VAR_ADD) {
                    // Split the variable, to get its name and the value we want to add.
                    let variable_split = template
                        .apply_path
                        .split_at(template.apply_path.find(VAR_ADD).unwrap()); // unwrap is ok, since we know that VAR_ADD is present.
                    let variable_name =
                        variable_split.0.strip_prefix(VAR_PREF).unwrap().to_string(); // same here.
                    let additional_value = variable_split.1.strip_prefix(VAR_ADD).unwrap(); // and  here.

                    if let Some(variable) = self_clone.get_var(&variable_name) {
                        // here we add the additional value
                        let mut value = variable.value[0].clone(); // here we clone, because we don't want to add the value to the variable permanently
                        value.push_str(additional_value);
                        let variable_path_buf = path::PathBuf::from(&value);
                        if variable_path_buf.is_absolute() {
                            new_template.apply_path = value;
                        } else {
                            let mut canonicalized_apply_path =
                                self.path.clone().parent().unwrap().to_path_buf();
                            canonicalized_apply_path.push(value);
                            new_template.apply_path =
                                canonicalized_apply_path.to_str().unwrap().to_string();
                        }
                    }
                } else {
                    let variable_name = template
                        .apply_path
                        .strip_prefix(VAR_PREF)
                        .unwrap()
                        .to_string();

                    if let Some(variable) = self_clone.get_var(&variable_name) {
                        let variable_path_buf = path::PathBuf::from(&variable.value[0]);
                        if variable_path_buf.is_absolute() {
                            new_template.apply_path = variable.value[0].clone();
                        // only take the first value, since we only accpet one apply path now
                        } else {
                            let mut canonicalized_apply_path =
                                self.path.clone().parent().unwrap().to_path_buf();
                            canonicalized_apply_path.push(variable.value[0].clone());
                            new_template.apply_path =
                                canonicalized_apply_path.to_str().unwrap().to_string();
                        }
                    }
                }
            } else {
                let variable_path_buf = path::PathBuf::from(&template.apply_path);
                if variable_path_buf.is_absolute() {
                    new_template.apply_path = template.apply_path.clone();
                } else {
                    let mut canonicalized_apply_path =
                        self.path.clone().parent().unwrap().to_path_buf();
                    canonicalized_apply_path.push(template.apply_path.clone());
                    new_template.apply_path = normalize_path(&canonicalized_apply_path)
                        .to_str()
                        .unwrap()
                        .to_string();
                    new_template.apply_path.push('/');
                }
            }

            let mut hostname_clone: Vec<String> = Vec::new();
            if template.hostnames.is_empty() {
                new_template.hostnames = default_template.clone().hostnames;
            } else {
                for hostname_entry in template.hostnames.iter() {
                    if hostname_entry.starts_with(VAR_PREF) {
                        if let Some(variable) =
                            self_clone.get_var(&hostname_entry.strip_prefix(VAR_PREF).unwrap())
                        {
                            let mut variable_value = variable.value.clone();
                            hostname_clone.append(&mut variable_value);
                        }
                    } else {
                        hostname_clone.push(hostname_entry.to_string())
                    }
                }
                new_template.hostnames = hostname_clone;
            }

            let mut extensions_clone: Vec<String> = Vec::new();
            if template.extensions.is_empty() {
                new_template.extensions = default_template.clone().extensions;
            } else {
                for extension in template.extensions.iter() {
                    if extension.starts_with(VAR_PREF) {
                        if extension.contains(VAR_ADD) {
                            let split_variable =
                                extension.split_at(extension.find(VAR_ADD).unwrap());
                            let variable_name =
                                split_variable.0.strip_prefix(VAR_PREF).unwrap().trim();
                            let additional_value = split_variable.1.strip_prefix(VAR_ADD).unwrap();
                            if let Some(variable) = self_clone.get_var(&variable_name) {
                                let mut variable_value = variable.value.clone();
                                variable_value.push(additional_value.to_string());
                                extensions_clone.append(&mut variable_value)
                            }
                        } else if let Some(variable) =
                            self_clone.get_var(&extension.strip_prefix(VAR_PREF).unwrap())
                        {
                            let mut variable_value = variable.value.clone();
                            extensions_clone.append(&mut variable_value)
                        }
                    } else {
                        extensions_clone.push(extension.to_string())
                    }
                }
                new_template.extensions = extensions_clone;
            }

            if (self_clone.empty_fields & 0b00001000) == 0 {
                new_template.recursive = default_template.recursive;
            } else {
                new_template.recursive = template.recursive
            }

            if (self_clone.empty_fields & 0b00010000) == 0 {
                new_template.parse_files = default_template.parse_files
            } else {
                new_template.parse_files = template.parse_files;
            }
        }
        new_derfile.vars = self.vars.clone();
        new_derfile.path = self.path.clone();

        if debug() {
            println!(
                "[\x1b[32mINFO\x1b[0m] Parsed derfile {}:\n{}",
                self.path.to_str().unwrap(),
                new_derfile
            )
        };

        new_derfile
    }

    /// Load a derfile from disk.
    pub fn load_derfile(buffer: String, path: &path::Path, config: &Config) -> Result<Self> {
        let mut derfile: Derfile = Derfile {
            path: path.to_path_buf(),
            ..Default::default()
        };
        let lines = buffer.lines();

        // list of template definitions(lines that start with "[" and end with "]")
        let template_indecies: Vec<usize> = lines
            .clone()
            .enumerate()
            .filter(|line| {
                line.1.trim().starts_with(TEMPLATE_LEFT) && line.1.trim().ends_with(TEMPLATE_RIGHT)
            })
            .map(|each| each.0)
            .collect();

        let var_lines: Vec<String> = lines
            .clone()
            .enumerate()
            .filter(|line| line.1.trim().starts_with(VAR_PREF))
            .map(|each| each.1.to_string())
            .collect();

        // [x] variables from shell code
        // [x] stuff such as "echo $PATH" does not work
        // E.i. instead of returning the value
        // of environmental variable $PATH(which could look like:
        // /home/user/.local/bin/:/usr/bin/:/usr/local/bin/ etc.) we just get the actual string
        // "$PATH".
        // I have found no solution for this, you might look into this and submit a PR and make me feel
        // like a dumb ass(which would be highly appreciated). But if this worked, I would be delighted.
        // What we do instead, is have a special keyword for returning environmental variables. So
        // something like this: "env`$PATH`" could return the actual value of our environmental
        // variable.
        for line in var_lines.iter() {
            if line.contains('=') {
                let split = line.split_at(line.find('=').unwrap());
                let name = split.0.trim().strip_prefix(VAR_PREF).unwrap().to_string();
                let mut value: Vec<String> = Vec::new();
                let right_side = split.1.trim().strip_prefix('=').unwrap().trim().to_string();

                if right_side.starts_with(CODE_SEP) {
                    if let Some(index) = right_side.find(CODE_SEP) {
                        let content = right_side[index + 1..right_side.len() - 1].to_string();
                        value = vec![execute_code(content.clone()).map_err(|_| {
                            format!("Unable to execute code inside a code block: {content}")
                        })?];
                    }
                } else if right_side.contains(',') {
                    let split: Vec<String> = right_side
                        .split(',')
                        .map(|x| x.trim().to_string())
                        .collect();
                    value = split
                } else {
                    value = vec![right_side.clone()]
                }

                // Special keywords are defined in const CODE_KEYWORDS and each of them is manually
                // implemented here. So far, there's only "env", which is used to retrieve environmental
                // variables
                for each in CODE_KEYWORDS {
                    if right_side.starts_with(each) && each == "env" {
                        if let Some(index) = right_side.find(CODE_SEP) {
                            let env_variable = &right_side[index + 1..right_side.len() - 1];
                            if debug() {
                                println!(
                                    "[\x1b[32mINFO\x1b[0m] Environmental variable accessed: {env_variable}",
                                );
                            }

                            if let Ok(env_output) = env::var(env_variable) {
                                value = vec![env_output]
                            }
                        }
                    }
                }

                derfile.add_var(name, value)
            }
        }

        let lines: Vec<String> = lines.clone().map(|x| x.to_string()).collect();
        for (ii, index) in template_indecies.iter().enumerate() {
            let template_lines: Vec<String> =
                if template_indecies.len() == 1 || ii == template_indecies.len() - 1 {
                    lines.clone().drain(index..).collect()
                } else {
                    lines
                        .clone()
                        .drain(index..&template_indecies[ii + 1])
                        .collect()
                };

            let mut template_name: String =
                template_lines[0][1..template_lines[0].len() - 1].to_string();
            for line in template_lines.iter() {
                if line.starts_with(TEMPLATE_LEFT) && line.ends_with(TEMPLATE_RIGHT) {
                    let mut derfile_dir_path =
                        path.to_owned().clone().parent().unwrap().to_path_buf();
                    derfile_dir_path.push(&template_name);
                    template_name = derfile_dir_path.to_str().unwrap().to_string();

                    derfile.add_template(template_name.clone());
                    if let Some(template) = derfile.get_template(&template_name) {
                        template.set_name(template_name.clone())
                    }
                }

                if line.is_empty() {
                    continue;
                }

                if line.contains('=') {
                    let split = line.split_at(line.find('=').unwrap());
                    match split.0.trim() {
                        "final_name" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let final_name = split.1.strip_prefix('=').unwrap().trim();
                                table.set_final_name(final_name.to_string());
                                derfile.empty_fields |= 0b00000001;
                            }
                        }
                        "hostnames" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                if split.1.contains(',') {
                                    let value_list: Vec<&str> = split.1.split(',').collect();
                                    for each in value_list {
                                        if let Some(s) = each.strip_prefix('=') {
                                            table.add_hostname(s.trim().to_string())
                                        } else {
                                            table.add_hostname(each.trim().to_string())
                                        }
                                    }
                                    derfile.empty_fields |= 0b00000010;
                                } else {
                                    table.add_hostname(
                                        split.1.strip_prefix('=').unwrap().trim().to_string(),
                                    );
                                    derfile.empty_fields |= 0b00000010;
                                }
                            }
                        }
                        "apply_path" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let apply_path = split.1.strip_prefix('=').unwrap().trim();
                                table.set_apply_path(apply_path.to_string());
                                derfile.empty_fields |= 0b00000100;
                            }
                        }
                        "recursive" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let recursive_field = split.1.strip_prefix('=').unwrap().trim();
                                if recursive_field == "true" {
                                    table.set_recursive(true)
                                } else {
                                    table.set_recursive(false)
                                }
                                derfile.empty_fields |= 0b00001000;
                            }
                        }
                        "parse_files" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let field = split.1.strip_prefix('=').unwrap().trim();
                                if field == "true" {
                                    table.set_parse_files(true)
                                } else {
                                    table.set_parse_files(false)
                                }
                                derfile.empty_fields |= 0b00010000;
                            }
                        }
                        "extensions" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let field = split.1.strip_prefix('=').unwrap().trim();
                                if field.contains(',') {
                                    for split_component in field.split(',') {
                                        // println!("{}", split_component.trim());
                                        table.add_extension(split_component.trim().to_string());
                                    }
                                    derfile.empty_fields |= 0b00100000
                                } else {
                                    table.add_extension(field.trim().to_string());
                                    derfile.empty_fields |= 0b00100000;
                                }
                            }
                        }
                        some => {
                            if some.starts_with(VAR_PREF) {
                                continue;
                            } else {
                                eprintln!("WARN: {} is not a valid template filed!", split.0)
                            }
                        }
                    }
                }
            }
        }
        let mut d = derfile.clone();
        d.with_config(config);
        derfile = d.parse();

        Ok(derfile)
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{}]", self.name)?;
        writeln!(f, "apply_path = {}", self.apply_path)?;
        writeln!(f, "final_name = {}", self.final_name)?;
        writeln!(f, "hostnames = {}", self.serialize_hostnames())?;
        writeln!(f, "recursive = {}", self.recursive)?;
        writeln!(f, "parse_files = {}", self.parse_files)?;
        writeln!(f, "extensions = {}", self.serialize_extensions())
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "${} = {:?}", self._name, self.serialize())
    }
}

impl fmt::Display for Derfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for template in self.templates.iter() {
            writeln!(f, "{}", template.1)?;
        }

        for variable in self.vars.iter() {
            writeln!(f, "{}", variable.1)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn load_and_parse_derfile() {
        let derfile_string = r"#
$host = `hostnamectl hostname`
$out = some/out/path/

[some/name.t]
final_name = name
apply_path = $out
hostnames = $host

[some/dir]
final_name = outdir
apply_path = $out:dirs/
hostnames = $host
parse_files = false
extensions = t
recursive = true
            "
        .to_string();
        let derfile_result =
            Derfile::load_derfile(derfile_string, Path::new("some_path"), &Config::default());

        assert_eq!(derfile_result.is_ok(), true);
    }

    #[test]
    fn serialization_test() {
        let derfile_string = r"#
$host = some, real, weird, hostnames

[some/name.t]
final_name = name
apply_path = some/path/
hostnames = $host
extensions = t, g, h
            "
        .to_string();
        let derfile_result =
            Derfile::load_derfile(derfile_string, Path::new("some_path"), &Config::default())
                .unwrap();
        let template = derfile_result
            .templates
            .iter()
            .filter(|t| t.0 != "[default-template]")
            .last()
            .unwrap()
            .1;
        let variable = derfile_result.vars.iter().last().unwrap().1;

        assert_eq!(
            template.serialize_hostnames(),
            "some,real,weird,hostnames".to_string()
        );
        assert_eq!(template.serialize_extensions(), "t,g,h".to_string());
        assert_eq!(
            variable.serialize(),
            "some, real, weird, hostnames".to_string()
        );
    }
}
