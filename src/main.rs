use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::process;

type Args = Vec<Arg>;

enum Arg {
    Help,
    Derfile(String),
    Apply(String),
}

fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for qucik multisystem application of dotfiles, with template supporting.\n");
    println!("-a --apply PATH       apply dotfiles to a path.");
    println!("-f --file PATH        use a specified derfile.");
    println!("-h --help PATH        show this help message.")
}

#[derive(Debug, Clone, Default)]
struct Template {
    name: String,
    final_name: String,
    hostnames: Vec<String>,
    apply_path: String,
}

impl Template {
    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn set_final_name(&mut self, final_name: String) {
        self.final_name = final_name
    }

    fn add_hostname(&mut self, hostname: String) {
        self.hostnames.push(hostname)
    }

    fn set_apply_path(&mut self, apply_path: String) {
        self.apply_path = apply_path
    }
}

#[derive(Debug, Clone, Default)]
struct Variable {
    _name: String,
    value: Vec<String>,
}

impl Variable {
    fn new(_name: String, value: Vec<String>) -> Self {
        Self { _name, value }
    }
}

#[derive(Debug, Clone, Default)]
struct Derfile {
    templates: HashMap<String, Template>,
    vars: HashMap<String, Variable>,
}

impl Derfile {
    fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    fn get_template(&mut self, name: &String) -> Option<&mut Template> {
        self.templates.get_mut(name)
    }

    fn add_var(&mut self, name: String, value: Vec<String>) {
        self.vars.insert(name.clone(), Variable::new(name, value));
    }

    fn get_var(&mut self, name: &String) -> Option<&mut Variable> {
        self.vars.get_mut(name)
    }

    fn parse(&mut self) -> Self {
        let mut derfile: Derfile = Default::default();
        let mut self_clone = self.clone();
        for (template_name, template) in self.templates.iter() {
            derfile.add_template(template_name.clone());
            let mut temp = derfile.get_template(&template_name).unwrap();
            temp.name = template_name.to_string();
            if template.final_name.starts_with("$") {
                let variable_name = template.final_name.strip_prefix("$").unwrap().to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut temp = derfile.get_template(&template.name).unwrap();
                    temp.final_name = variable.value[0].clone(); // only take the fist value, sicne we only accept only one final file name
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.final_name = template.final_name.clone();
            }
            if template.apply_path.starts_with("$") {
                let variable_name = template.apply_path.strip_prefix("$").unwrap().to_string();

                if let Some(variable) = self_clone.get_var(&variable_name) {
                    let mut template = derfile.get_template(&template.name).unwrap();
                    template.apply_path = variable.value[0].clone(); // only take the first value, since we only accpet one apply path now
                }
            } else {
                let mut temp = derfile.get_template(&template_name).unwrap();
                temp.apply_path = template.apply_path.clone()
            }
            let mut hostname_clone: Vec<String> = Vec::new();
            for hostname_entry in template.hostnames.iter() {
                if hostname_entry.starts_with("$") {
                    if let Some(variable) =
                        self_clone.get_var(&hostname_entry.strip_prefix("$").unwrap().to_string())
                    {
                        let mut variable_value = variable.value.clone();
                        hostname_clone.append(&mut variable_value);
                    }
                } else {
                    hostname_clone.push(hostname_entry.to_string())
                }
            }

            let template = derfile.get_template(&template.name).unwrap();
            template.hostnames = hostname_clone
        }
        derfile.vars = self.vars.clone();

        derfile
    }
}

fn parse_args(args: Vec<String>) -> Args {
    let mut ret = Vec::new();

    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => {
                ret.push(Arg::Help);
            }
            "-f" | "--file" => {
                ret.push(Arg::Derfile(args[i + 1].clone()));
            }
            "-a" | "--apply" => ret.push(Arg::Apply(args[i + 1].clone())),
            _ => (),
        }
    }

    ret
}

fn execute_code(command: String) -> Option<String> {
    let output: Option<String>;
    if command.contains(" ") {
        let split: Vec<&str> = command.split(" ").collect();
        let cmd = split[0];
        let args = &split[1..];

        let process = process::Command::new(cmd)
            .args(args)
            .output();
        if let Ok(out) = process {
            let str = std::str::from_utf8(&out.stdout);
            let str = str.expect("Unable to convert command output to string");
            output = Some(str.to_string())
        } else {
            output = Some("".to_string())
        }
    } else {
        let process = process::Command::new(command)
            .output();
        if let Ok(out) = process {
            let str = std::str::from_utf8(&out.stdout);
            let str = str.expect("Unable to convert command output to string");
            output = Some(str.to_string())
        } else {
            output = Some("".to_string())
        }

    }

    output
}

// TODO:
//  [x] templates
//  [x] variables ->
//      [ ] variable from code execution?
fn load_derfile(path: &path::Path) -> io::Result<Derfile> {
    let buffer = fs::read_to_string(path)?;
    let mut derfile: Derfile = Default::default();
    let lines = buffer.lines();

    // list of template definitions(lines that start with "[" and end with "]")
    let template_indecies: Vec<usize> = lines
        .clone()
        .enumerate()
        .filter(|line| line.1.trim().starts_with("[") && line.1.trim().ends_with("]"))
        .map(|each| return each.0)
        .collect();

    let var_lines: Vec<String> = lines
        .clone()
        .enumerate()
        .filter(|line| line.1.trim().starts_with("$"))
        .map(|each| return each.1.to_string())
        .collect();

    // TODO
    // [ ] variables from shell code
    for line in var_lines.iter() {
        if line.contains("=") {
            let split = line.split_at(line.find("=").unwrap());
            let name = split.0.trim().strip_prefix("$").unwrap().to_string();
            let value: Vec<String>;
            let right_side = split
                .1
                .trim()
                .to_string()
                .strip_prefix("=")
                .unwrap()
                .trim()
                .to_string();

            if right_side.contains(",") {
                let split: Vec<String> = right_side
                    .split(",")
                    .map(|x| x.trim().to_string())
                    .collect();
                value = split
            } else {
                value = vec![right_side]
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

        let template_name: String = template_lines[0][1..template_lines[0].len() - 1].to_string();
        for line in template_lines.iter() {
            if line.starts_with("[") && line.ends_with("]") {
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
                            table.set_final_name(
                                split
                                    .1
                                    .to_string()
                                    .strip_prefix("=")
                                    .unwrap()
                                    .to_string()
                                    .trim()
                                    .to_string(),
                            )
                        }
                    }
                    "hostnames" => {
                        if let Some(table) = derfile.get_template(&template_name) {
                            if split.1.contains(",") {
                                let value_list: Vec<String> =
                                    split.1.split(',').map(|x| x.to_string()).collect();
                                for each in value_list {
                                    if let Some(s) = each.strip_prefix("=") {
                                        table.add_hostname(s.to_string().trim().to_string())
                                    } else {
                                        table.add_hostname(each.trim().to_string())
                                    }
                                }
                            } else {
                                table.add_hostname(
                                    split
                                        .1
                                        .to_string()
                                        .strip_prefix("=")
                                        .unwrap()
                                        .to_string()
                                        .trim()
                                        .to_string(),
                                )
                            }
                        }
                    }
                    "apply_path" => {
                        if let Some(table) = derfile.get_template(&template_name) {
                            table.set_apply_path(
                                split
                                    .1
                                    .to_string()
                                    .strip_prefix("=")
                                    .unwrap()
                                    .to_string()
                                    .trim()
                                    .to_string(),
                            )
                        }
                    }
                    some => {
                        if some.starts_with("$") {
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

fn run(args: Args) -> io::Result<()> {
    for arg in args {
        match arg {
            Arg::Derfile(file) => {
                let df = load_derfile(path::Path::new(&file)).unwrap();
                println!("{:#?}", df);
                // TODO: not sure if it should work like this
                // or if the vars substitution should be done while parsing
                // let df = Derfile::apply(df);
            }
            // TODO: this might be useless
            Arg::Apply(_path) => {
                let derfile = path::Path::new("./derfile");

                if !derfile.exists() {
                    return Ok(());
                }

                // let loaded = load_derfile(derfile)?;
            }
            Arg::Help => {
                help_function();
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let parsed_args = parse_args(args);
    run(parsed_args)?;

    Ok(())
}
