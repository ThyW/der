use std::collections::HashMap;
use std::env;
use std::path;
use std::process;

mod derfile;
mod error;
mod template;
mod utils;

use derfile::*;
use error::*;
use template::*;

/// Wrapper type for a list of parsed command line arguments
type Args = Vec<Arg>;

/// Representation of command line arguments
enum Arg {
    Help,
    Derfile(String),
    // Debug(Vec<String>),
    Apply,
}

/// Help fucntion to be diplayed when the `-h` or `--help` flags are passed
fn help_function() {
    println!("der v0.1");
    println!("author: J. Kapko <kamo.bavmesa@gmail.com>");
    println!("about: der is a tool for qucik multisystem application of dotfiles, with template supporting.\n");
    println!("-a --apply PATH                                           Apply using  specified path to derfile.");
    println!("-f --file  PATH                                           Use a specified derfile.");
    // println!("-d --debug PATH FINAL_NAME APPLY_PATH [HOSTNAMES]         Use a specified derfile.");
    println!("-h --help  PATH                                           Show this help message.")
}

/// Pasre command line arguments
fn parse_args(args: Vec<String>) -> Args {
    // return vector of Args
    let mut ret = Vec::new();

    // go through command line arguments and parse them accordingly
    for (i, entry) in args.iter().enumerate() {
        match &entry[..] {
            "-h" | "--help" => ret.push(Arg::Help),
            "-f" | "--file" => ret.push(Arg::Derfile(args[i + 1].clone())),
            // "-d" | "--debug" => ret.push(Arg::Debug(args[i + 1..].to_vec())),
            "-a" | "--apply" => ret.push(Arg::Apply),
            _ => (),
        }
    }

    ret
}

/// Attempt to execute shell code
fn execute_code(command: String) -> Result<String> {
    // Get a list of all environmental args.
    let vars: HashMap<String, String> = env::vars().collect();
    // Split the command into its components.
    if command.contains(" ") {
        let split: Vec<&str> = command.split(" ").collect();
        let cmd = split[0];
        let args = &split[1..];

        let output = process::Command::new(cmd).args(args).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect(&format!(
            "ERROR: Unable to convert command output to string! [Error value: {}]",
            &command
        ));
        return Ok(str.trim().to_string());
    } else {
        let output = process::Command::new(&command).envs(&vars).output()?;

        let str = std::str::from_utf8(&output.stdout);
        let str = str.expect(&format!(
            "ERROR: Unable to convert command output to string! [Error value: {}]",
            &command
        ));
        return Ok(str.trim().to_string());
    }
}

/// Parse arguments and run the application
fn run(args: Args) -> Result {
    let mut derfile: Option<Derfile> = None;
    for arg in args {
        match arg {
            // Specifiy a derfile to be used.
            Arg::Derfile(file) => {
                // Get an absolute path to derfile.
                derfile = Some(derfile::Derfile::load_derfile(
                    &path::Path::new(&file).canonicalize().unwrap(),
                )?);
            }

            // Apply template files according to derfile rules.
            Arg::Apply => {
                let derfile_default_path = path::Path::new("./derfile").canonicalize();

                if !derfile_default_path.is_ok() && derfile.clone().is_none() {
                    return Err("Error: No derfile path specified or present!".into());
                }

                if derfile.is_none() {
                    derfile = Some(derfile::Derfile::load_derfile(&derfile_default_path?)?);
                }

                let template_structures: Vec<Template> = derfile
                    .clone()
                    .unwrap()
                    .templates
                    .values()
                    .map(Clone::clone)
                    .collect();
                let vecs = recursive_build(template_structures)?;
                for structure in vecs {
                    match structure {
                        TemplateStructure::File(mut f) => {
                            f.apply()?;
                        }
                        // This can probably be removed, since we do directory creations when
                        // applying tempalte files.
                        TemplateStructure::Directory(_) => {}
                    }
                }
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

    println!("Success");

    Ok(())
}
