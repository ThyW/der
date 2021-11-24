use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;

use crate::error::*;
use crate::execute_code;
use crate::utils::*;

/// Symbols for derfile parsing, these can be changed before compilation.
pub const TEMPLATE_LEFT: &str = "[";
pub const TEMPLATE_RIGHT: &str = "]";
pub const CODE_SEP: &str = "`";
pub const VAR_PREF: &str = "$";
pub const VAR_ADD: &str = ":"; // code separator for adding other values
pub const CODE_KEYWORDS: [&str; 1] = ["env"];

/// Representation of a single template secition in a derfile.
#[derive(Debug, Clone, Default)]
pub struct Template {
    pub name: String,
    pub final_name: String,
    pub hostnames: Vec<String>,
    pub apply_path: String,
    pub parse_files: bool,
    pub extensions: Vec<String>,
    pub recursive: bool,
    // pub keep_structure: bool,
}

/// Representation of a single variable in a derfile.
#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub _name: String,
    pub value: Vec<String>,
}

/// Representation of a derfile.
/// `templates`: HashMap of templates and their names.
/// `vars`: HashMap of variables and their names.
/// `path`: Absolute path to a derfile.
#[derive(Debug, Clone, Default)]
pub struct Derfile {
    pub templates: HashMap<String, Template>,
    pub vars: HashMap<String, Variable>,
    path: path::PathBuf,
}

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

    /* pub fn set_keep_structure(&mut self, arg: bool) {
        self.keep_structure = arg
    } */

    pub fn add_extension(&mut self, ext: String) {
        self.extensions.push(ext)
    }
}

impl Variable {
    fn new(_name: String, value: Vec<String>) -> Self {
        Self { _name, value }
    }
}

impl Derfile {
    pub fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    pub fn get_template(&mut self, name: &String) -> Option<&mut Template> {
        self.templates.get_mut(name)
    }

    pub fn add_var(&mut self, name: String, value: Vec<String>) {
        self.vars.insert(name.clone(), Variable::new(name, value));
    }

    pub fn get_var(&mut self, name: &String) -> Option<&mut Variable> {
        self.vars.get_mut(name)
    }

    // TODO: Add support for variable concatenation and directory parsing
    pub fn parse(self) -> Self {
        let mut new_derfile: Derfile = Default::default();
        let mut self_clone = self.clone();
        for (template_name, template) in self.templates.iter() {
            new_derfile.add_template(template_name.clone());
            let mut new_template = new_derfile.get_template(&template_name).unwrap();
            new_template.name = template_name.to_string();
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
                        new_template.final_name = variable.value[0].clone(); // only take the fist value, sicne we only accept one final file name
                    }
                }
            } else {
                new_template.final_name = template.final_name.clone();
            }
            if template.apply_path.starts_with(VAR_PREF) {
                if template.apply_path.contains(VAR_ADD) {
                    let variable_split = template
                        .apply_path
                        .split_at(template.apply_path.find(VAR_ADD).unwrap());
                    let variable_name =
                        variable_split.0.strip_prefix(VAR_PREF).unwrap().to_string();
                    let additional_value = variable_split.1.strip_prefix(VAR_ADD).unwrap();

                    if let Some(variable) = self_clone.get_var(&variable_name) {
                        // here we add the additional value
                        let mut value = variable.value[0].clone();
                        value.push_str(additional_value);
                        let variable_path_buf = path::PathBuf::from(&value);
                        if variable_path_buf.is_absolute() {
                            new_template.apply_path = value; // only take the first value, since we only accpet one apply path now
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
                            new_template.apply_path = variable.value[0].clone(); // only take the first value, since we only accpet one apply path now
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
            for hostname_entry in template.hostnames.iter() {
                if hostname_entry.starts_with(VAR_PREF) {
                    if let Some(variable) = self_clone
                        .get_var(&hostname_entry.strip_prefix(VAR_PREF).unwrap().to_string())
                    {
                        let mut variable_value = variable.value.clone();
                        hostname_clone.append(&mut variable_value);
                    }
                } else {
                    hostname_clone.push(hostname_entry.to_string())
                }
            }

            new_template.hostnames = hostname_clone;

            let mut extensions_clone: Vec<String> = Vec::new();
            for extension in template.extensions.iter() {
                if extension.starts_with(VAR_PREF) {
                    if extension.contains(VAR_ADD) {
                        let split_variable = extension.split_at(extension.find(VAR_ADD).unwrap());
                        let variable_name = split_variable.0.strip_prefix(VAR_PREF).unwrap().trim();
                        let additional_value = split_variable.1.strip_prefix(VAR_ADD).unwrap();
                        if let Some(variable) = self_clone.get_var(&variable_name.to_string()) {
                            let mut variable_value = variable.value.clone();
                            variable_value.push(additional_value.to_string());
                            extensions_clone.append(&mut variable_value)
                        }
                    } else {
                        if let Some(variable) = self_clone
                            .get_var(&extension.strip_prefix(VAR_PREF).unwrap().to_string())
                        {
                            let mut variable_value = variable.value.clone();
                            extensions_clone.append(&mut variable_value)
                        }
                    }
                } else {
                    extensions_clone.push(extension.to_string())
                }
            }
            new_template.extensions = extensions_clone;
            let cloned = self_clone.get_template(&template_name).unwrap();
            new_template.recursive = cloned.recursive;
            new_template.parse_files = cloned.parse_files;
            // t.keep_structure = cloned.keep_structure;
            println!("{:#?}", new_template);
        }
        new_derfile.vars = self.vars.clone();
        new_derfile.path = self.path.clone();

        if debug() {println!("{:#?}", new_derfile)};

        new_derfile
    }

    // TODO:
    //  [x] templates
    //  [x] variables
    //  [x] variable from code execution?
    //  [ ] find out what's wrong with it, it for example can't access environmental variables
    pub fn load_derfile(path: &path::Path) -> Result<Self> {
        let buffer = fs::read_to_string(path)?;
        let mut derfile: Derfile = Default::default();
        derfile.path = path.to_path_buf();
        let lines = buffer.lines();

        // list of template definitions(lines that start with "[" and end with "]")
        let template_indecies: Vec<usize> = lines
            .clone()
            .enumerate()
            .filter(|line| {
                line.1.trim().starts_with(TEMPLATE_LEFT) && line.1.trim().ends_with(TEMPLATE_RIGHT)
            })
            .map(|each| return each.0)
            .collect();

        let var_lines: Vec<String> = lines
            .clone()
            .enumerate()
            .filter(|line| line.1.trim().starts_with(VAR_PREF))
            .map(|each| return each.1.to_string())
            .collect();

        // TODO
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
            if line.contains("=") {
                let split = line.split_at(line.find("=").unwrap());
                let name = split.0.trim().strip_prefix(VAR_PREF).unwrap().to_string();
                let mut value: Vec<String> = Vec::new();
                let right_side = split.1.trim().strip_prefix("=").unwrap().trim().to_string();

                if right_side.starts_with(CODE_SEP) {
                    if let Some(index) = right_side.find(CODE_SEP) {
                        let content = right_side[index + 1..right_side.len() - 1].to_string();
                        value = vec![execute_code(content.clone())
                            .expect("Error: unablet to parse code block.")];
                    }
                } else if right_side.contains(",") {
                    let split: Vec<String> = right_side
                        .split(",")
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
                    if right_side.starts_with(each) {
                        match each {
                            "env" => {
                                if let Some(index) = right_side.find(CODE_SEP) {
                                    let env_variable = &right_side[index + 1..right_side.len() - 1];
                                    println!("{}", env_variable);

                                    if let Ok(env_output) = env::var(env_variable) {
                                        value = vec![env_output]
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                derfile.add_var(name, value)
            }
        }

        let lines: Vec<String> = lines.clone().map(|x| x.to_string()).collect();
        for (ii, index) in template_indecies.iter().enumerate() {
            let template_lines: Vec<String>;

            // TODO: inefficient? shouldn't clone on each iteration,
            // but maybe its okay since its dropped before the next iteration?
            if template_indecies.len() == 1 {
                template_lines = lines.clone().drain(index..).collect();
            } else if ii == template_indecies.len() - 1 {
                template_lines = lines.clone().drain(index..).collect();
            } else {
                template_lines = lines
                    .clone()
                    .drain(index..&template_indecies[ii + 1])
                    .collect();
            }

            let mut template_name: String =
                template_lines[0][1..template_lines[0].len() - 1].to_string();
            for line in template_lines.iter() {
                if line.starts_with(TEMPLATE_LEFT) && line.ends_with(TEMPLATE_RIGHT) {
                    // TODO: template name should be turned into absolute template path
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

                if line.contains("=") {
                    let split = line.split_at(line.find("=").unwrap());
                    match split.0.trim() {
                        "final_name" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let final_name = split.1.strip_prefix("=").unwrap().trim();
                                table.set_final_name(final_name.to_string())
                            }
                        }
                        "hostnames" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                if split.1.contains(",") {
                                    let value_list: Vec<&str> = split.1.split(',').collect();
                                    for each in value_list {
                                        if let Some(s) = each.strip_prefix("=") {
                                            table.add_hostname(s.trim().to_string())
                                        } else {
                                            table.add_hostname(each.trim().to_string())
                                        }
                                    }
                                } else {
                                    table.add_hostname(
                                        split.1.strip_prefix("=").unwrap().trim().to_string(),
                                    )
                                }
                            }
                        }
                        "apply_path" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let apply_path = split.1.strip_prefix("=").unwrap().trim();
                                table.set_apply_path(apply_path.to_string())
                            }
                        }
                        "recursive" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let recursive_field = split.1.strip_prefix("=").unwrap().trim();
                                if recursive_field == "true" {
                                    table.set_recursive(true)
                                } else {
                                    table.set_recursive(false)
                                }
                            }
                        }
                        "parse_files" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let field = split.1.strip_prefix("=").unwrap().trim();
                                if field == "true" {
                                    table.set_parse_files(true)
                                } else {
                                    table.set_parse_files(false)
                                }
                            }
                        }
                        /* "keep_structure" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let field = split.1.strip_prefix("=").unwrap().trim();
                                if field == "true" {
                                    table.set_keep_structure(true)
                                } else {
                                    table.set_keep_structure(false)
                                }
                            }
                        } */

                        "extensions" => {
                            if let Some(table) = derfile.get_template(&template_name) {
                                let field = split.1.strip_prefix("=").unwrap().trim();
                                if field.contains(",") {
                                    for split_component in field.split(",") {
                                        // println!("{}", split_component.trim());
                                        table.add_extension(split_component.trim().to_string())
                                    }
                                } else {
                                    table.add_extension(field.trim().to_string())
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

        Ok(derfile.parse())
    }
}
