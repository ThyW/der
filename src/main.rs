use std::collections::HashMap;
use std::env;
use std::path;
use std::process;

mod error;
mod derfile;
mod template;

use derfile::*;
use template::*;
use error::*;

/// Wrapper type for a list of parsed command line arguments
type Args = Vec<Arg>;

/// Representation of command line arguments
enum Arg {
    Help,
    Derfile(String),
    Debug(Vec<String>),
    Apply,
}


/// Help fucntion to be diplayed when the `-h` or `--help` flags are passed
fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for qucik multisystem application of dotfiles, with template supporting.\n");
    println!("-a --apply PATH                                           Apply using  specified path to derfile.");
    println!("-f --file  PATH                                           Use a specified derfile.");
    println!("-d --debug PATH FINAL_NAME APPLY_PATH [HOSTNAMES]         Use a specified derfile.");
    println!("-h --help  PATH                                           Show this help message.")
}


/// Pasre command line arguments
fn parse_args(args: Vec<String>) -> Args {
    let mut ret = Vec::new();

    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => ret.push(Arg::Help),
            "-f" | "--file" => ret.push(Arg::Derfile(args[i + 1].clone())),
            "-d" | "--debug" => ret.push(Arg::Debug(args[i + 1..].to_vec())),
            "-a" | "--apply" => ret.push(Arg::Apply),
            _ => (),
        }
    }

    ret
}

/// Attempt to execute shell code
fn execute_code(command: String) -> Result<String> {
    let vars: HashMap<String, String> = env::vars().collect();
    if command.contains(" ") {
        let split: Vec<&str> = command.split(" ").collect();
        let cmd = split[0];
        let args = &split[1..];

        let output = process::Command::new(cmd).args(args).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect("ERROR: Unable to convert command output to string");
        return Ok(str.to_string().trim().to_string());
    } else {
        let output = process::Command::new(command).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect("ERROR: Unable to convert command output to string.");
        return Ok(str.to_string().trim().to_string());
    }
}


/// Parse arguments and run the application
fn run(args: Args) -> Result {
    let mut derfile: Option<Derfile> = None;
    for arg in args {
        match arg {
            Arg::Derfile(file) => {
                derfile = Some(derfile::Derfile::load_derfile(path::Path::new(&file))?);
            }
            Arg::Apply => {
                let derfile_default_path = path::Path::new("derfile");

                if !derfile_default_path.exists() {
                    return Err("Error: No derfile path specified or present!".into());
                }

                if derfile.is_none() {
                    derfile = Some(derfile::Derfile::load_derfile(derfile_default_path)?);
                }

                let mut templates = derfile
                    .clone()
                    .unwrap()
                    .templates
                    .values()
                    .map(|t| t.clone().into())
                    .collect::<Vec<TemplateFile>>();
                for each_template in templates.iter_mut() {
                    each_template.apply()?;
                }
            }
            Arg::Debug(parse_args) => {
                // [x] TODO: test this please! [TESTING DONE]
                let hostnames = parse_args[3..].to_vec();
                let mut template_config = TemplateFile::new(
                    parse_args[0].clone(),
                    parse_args[1].clone(),
                    parse_args[2].clone(),
                    hostnames,
                );
                template_config.apply()?
            }
            Arg::Help => {
                help_function();
            }
        }
    }

    Ok(())
}

fn main() -> Result {
    let args: Vec<String> = env::args().collect();
    let parsed_args = parse_args(args);

    run(parsed_args)?;

    Ok(())
}
