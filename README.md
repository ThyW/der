# About
`der` is a helpful tool for managing your dotfiles across multiple machines. It comes in handy when you want to have multiple versions of the same config file, each for a different machine. Instead of having multiple copies of your config file, `der` allows you to have one single file, a template, in which you can define sections, which are specific to a single machine or to multiple machines. It can also work great with GNU Stow for quick application of your dotfiles(although, the same functionality can be achieved natively, without any symlinks).

# Installation
`der` is written in [Rust](https://rust-lang.org), so make sure you have `cargo` installed and up-to-date.
Once you clone the repository, you can just do:

```console
$ cargo install --path /somewhere/in/your/path 
```

Maybe also coming to some package managers soon. Who knows? ;)

# Usage
First, we start by defining our `derfile`. All things considered, a `derfile` is really similar to a Makefile, however you may find the syntax a bit off-putting as it is basically TOML with some \$ denoted variables and in-line shell code...
Let's get stared, shall we?

## Derfiles

### Variables
```
# Comments start with a hashtag
# Varialbes are defined like so:
# they are always treated like strings and they don't need to be surrouned by quotes
$variable = some
# you can also execute shell commands and set the variable's value like that:
$shell_code_var = `echo Hello`

# you can get the values of environmental variables like so:
$env_var = env`VARNAME`

# Appending to variables is also supported.
# For that we use the ':' right after the variable name.
# A few examples
$path = /home/user/some/dir
apply_path = $path:/next/dir/

$hosts = host1, host2, host3
hostnames = $hosts:,new_host, new_host_2

# However, this does not change the variable only uses its value and adds to it.
# So in the above example the value of 'apply_path' will be '/home/user/some/dir/next/dir/'
# and the value of 'hostnames' will be `host1, host2, host3, new_host, new_host_2`
```

### Templates
```
# Templates are parts of derfile and are defined as follows(example is a config for the Alacritty terminal emulator):

# Template name is a path (absolute or relative) to the template file.
[path/to/alacritty.yml.t] 
# The 'final_name' field is the name of the created parsed derfile.
# This can be any string you want.
final_name = alacritty.yml
# A list of comma separated values, hostnames, for which this file should be parsed.
hostnames = machine-one, machine-two, machine-three
# Apply path, in other words, where should our output file be placed.
# If the path does not exist, der will attepmt to create it!
# WARNING: This has to be a directory.
appy_path = /home/user/.config/alacritty/
```

Templates can also point to directories, which can contain template files. These templates have a few more options, such as, if the files should even be attempted to be parsed, which file extensions to look for within the files and a couple of other options.

```
[some/dir]
final_name = outdir
apply_path = out/
hostnames = $hosts
# A list of comma separated values, file name extensions to look for as template files.
extensions = t, tmp, template, tpl
# This is a boolean field, so only 'true' and 'false' will be accepted as valid values.
# If neither true nor false is found, the defaultt is assumed to be false!
# This option specifies, whether the files in the directory should be attempted to be parsed,
# if 'false', all files will just be placed to the output directory without any change.
parse_files = true
# recursive indicates whether or not all directories should be visited and parsed.
# If false, only the first directory will be visited and parsed, all the subdirectories will be ignored. 
# Again, a boolean value, so valid values are either true, or false.
recursive = true
```

### Templates with variables
```
$hosts = hostname1, hostname2, hostname3
$path = /some/path/to/output/the/template/to/

[some_config.conf.t]
final_name = some_config.conf
hostnames = $hosts
apply_path = $path
```

## Template files
Template files are really just your config file, with parts added for `der` to distinguish, which parts should be put on which machine.

### Syntax
Template files are made up of normal text(parts of the file, which will be included on every machine) and, so called `substitution blocks`. Substitution blocks are parts of the file which get included or excluded from the file, depending on the machine, or the hostname for that fact. Substitution blocks look like this:

```
# Substitution blocks are declared like so, '@@' followed by a list of comma separated values.
# These values are hostnames for which the code blocks should be included in the output file.
@@ hostname1,hostname2,hostname3 
your code goes, here, 
any number of lines
or spaces, or anything
@!
# Substitution blocks end with a '@!'. Both '@@' and '@!' must be on a new line.
```

### Example
All my dotfiles have be rewritten for usage with `der`, so if you seek further information or inspiration, feel free to check them out [here](https://gitea.redalder.org/ThyW/dotfiles).

# Why?
Why not? I've always wanted to use something with the same functionality as described above and since I had some time on my hands, I decided to write it myself, because I love coding and I love that you can both learn something new and relax while you're at it. So, it became my personal project for a while. I also wanted to write it in Rust as it's my favourite language and I love working with it. Another thing I wanted for the project, is to have no dependencies. I know some parts could have been done better and more elegantly, for example, use some normal format for writing derfiles instead of inventing my own. All in all, I'm really glad how it turned out and I hope it can be of use to someone else as well :-).

# Contributing
Open an issue, or submit a PR if you feel like it.
