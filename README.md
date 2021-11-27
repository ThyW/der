# About
`der` is a helpful tool for managing your dotfiles across multiple machines. It comes in handy when you want to have multiple versions of the same config file, but each for a different machine. Instead of having multiple copies of your single config file, `der` allows you to have one single file, a template, in which you can define sections, which are specific to a single machine or to multiple machines. It can also work great with GNU Stow for quick application of your dotfiles.

# Installation
`der` is written in [Rust](https://rust-lang.org), so make sure you have `cargo` installed and up-to-date.
Once you clone the repository, you can just do:

```console
$ cargo install --path /somewhere/in/your/path 
```

Maybe also coming to some package managers soon. Who knows? ;)

# Usage
First we start by defining our `derfile`. All things considered, a `derfile` is really similar to a Makefile, however you might find the syntax a bit off-putting as it is basically TOML with some \$ denoted variables and in line shell code...
So let's get stared, shall we?

## Derfiles

### Variables
```
# Comments start with a hashtag
# Varialbes are defined like so:
# they are always treated like strings and they don't need to have a " or ' around them
$variable = some
# you can also execute shell code and set it's output as a variable's value
$shell_code_var = `echo Hello`

# you can get environmental variables like so:
$env_var = env`VARNAME`

# Appending to variables is also supported.
# For that we use the : right after the variable name.
# A few examples
apply_path = $path:/next/dir/
hostnames = $hosts:new_host, new_host_2

# However, this does not change the variable only uses, its value and adds to it.
```

### Templates
```
# Templates are defined as follows(example is a config for the Alacritty terminal emulator):

# Template name is a path (absolute or relative) to the template file.
[path/to/alacritty.yml.t] 
# The final name is the name of the created parsed derfile.
# This can be any string you want.
final_name = alacritty.yml
# A list of comma separated values, hostnames, for which this file should be parsed.
hostnames = machine-one, machine-two, machine-three
# Apply path, in other words, where should our output file be placed.
# If the path does not exist, der will attepmt to create it!
# WARNING: This has to be a directory.
appy_path = /home/some/path/

```
Templates can also point to whole directories which can contain template files. These templates have a few more options, such as should the files even be attempted to be parsed, extensions to look for within the files and a few more.

```console
[some/dir]
final_name = outdir
apply_path = out/
hostnames = $hosts
# A list of comma separated values, file name extensions to look for as template files.
extensions = t, tmp, template, tpl
# This is a boolean field so either true or false is required here.
# If neither true nor false is found, the defaultt is assumed to be false!
parse_files = true
# Visit only the first directory, ignoring all subdirectories.
# Again, a boolean value, either true of false
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

### Example of der and stow working together
We could start by having an `install.sh` script in our dotfiles directory.
```
#!/bin/sh

# two steps
der -f ./derfile -a ./out/

# TODO: not tested
cd ./out/ && stow * 
```

Our dotfiels folder could look something likes this:
```
dotfiles/
    alacritty/
	alacritty.yml.t
    i3/
	config.t
    nvim/
	lua/
	    plugins.lua
	    keybinds.lua
	    settings.lua
	init.lua.t
    dunst/
	dunstrc.t
    bash/
	bashrc.t
	tmux.conf.t
```

We could then have our derfile laid out like so:
```
$out = out/

[alacritty/alacritty.yml.t]
final_name = alacritty.yml
hosntnames = $hostnames
apply_path = $out:alacritty/

[nvim]
final_name = nvim
hostnames = $hostnames
apply_path = $out
parse_files = false
extensions = t, tpl
recursive = true

[i3/config.t]
final_name = config
hostnames = $hostnames
apply_path = $out:i3/

[i3status-rust/config.toml]
final_name = config.toml
hostnames = $hostnames
apply_path = $out:i3status-rust/

[dunst/dunstrc.t]
final_name = dunstrc
honstames = $hostnames
apply_path = $out:dunst

$bash_out = $out:bash/

[bash/bashrc.t]
final_name = .bashrc
hostnames = $hostnames
apply_path = $bash_out

[bash/tmux.conf.t]
final_name = .tmux.conf
hostnames = $hostnames
apply_path = $bash_out
```

Then, if we run the script from `dotfiles/`, we should have all templates applied and `stow`ed to their exact locations.

## Template files
Template files are really just your config file, with parts added for `der` to distinguish, which parts should be put on which machine.

### Syntax
Template files are made up of normal text(parts of your configuration files, that you want to have on all the different machines) and so called `code blocks`. Code blocks are parts of the file which get included or excluded from the file, depending on the machine, or the hostname for that fact. Code blocks look like this:

```
# Code blocks are declared by so, two @ followed by a list of comma separated values.
# These values are hostnames for which the code blocks should be included in the output file.
@@ hostname1,hostname2,hostname3 
your code goes, here, 
any number of lines
or spaces, or anything
@!
# Code blocks end with a @ followed by !. These symbols must be on a new line.
```

### Example
A simple real world example would be something like this:
I have two machines that run [i3](https://i3wm.org), one is a laptop with one monitor(hostname: laptop), i.e. the main display and the other is my home laptop(hostname: desktop) with two monitors. What I want to achieve is to have a single line which tells i3 to execute an `xrandr` command only on my home laptop.
```
# file: config, an i3wm configuration file

# this is what we could do:
@@ desktop
exec --no-startup-id xrandr --output eDP-1 --off && xrandr --output HDMI-1 --primary  && xrandr --output DP-1 --right-of HDMI-1 move $ws1 to output HDMI-1 move $ws2 to output DP-1
@!
```
This solves the issue! If I clone my dotfiles, `der` them and check the output on both machines, on my home laptop I will be able to see the line which I wanted to have there and on the work laptop, the entire code block would be excluded.

# Contributing
Open an issue, or submit a PR if you feel like it.
