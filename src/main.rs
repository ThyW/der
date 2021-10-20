use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path;

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
    _value: String,
}

#[derive(Debug, Clone, Default)]
struct Derfile {
    templates: HashMap<String, Template>,
    _vars: HashMap<String, Variable>,
}

impl Derfile {
    fn add_template(&mut self, name: String) {
        self.templates.insert(name, Default::default());
    }

    fn get_template(&mut self, name: &String) -> Option<&mut Template> {
        self.templates.get_mut(name)
    }

    fn _add_var(&mut self, name: String) {
        self._vars.insert(name, Default::default());
    }

    fn _get_var(&mut self, name: &String) -> Option<&mut Variable> {
        self._vars.get_mut(name)
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

// TODO:
//  [x] tables
//  [ ] variables
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
        .collect::<Vec<(usize, &str)>>()
        .iter()
        .map(|each| return each.0)
        .collect();

    println!("{}", template_indecies.len());
    println!("{:?}", template_indecies);

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

        println!("{:?}", template_lines);

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
                    _ => {
                        eprintln!("WARN: {} is not a valid template filed!", split.0)
                    }
                }
            }
        }
    }

    Ok(derfile)
}

fn run(args: Args) -> io::Result<()> {
    for arg in args {
        match arg {
            Arg::Derfile(file) => {
                let df = load_derfile(path::Path::new(&file)).unwrap();
                println!("{:#?}", df)
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
