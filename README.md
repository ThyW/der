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
First we start by defining our `derfile`. In usage, a `derfile` is really similar to a Makefile, however you might find the syntax a bit off-putting as it is basically TOML with some \$ denoted variables and in line shell code...
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
```

### Templates
```
# templates are defined as follows(example is a config for the Alacritty terminal emulator):

# name, or in our case path, is relative to our derfile, e.g. the directory we envoke def from
[path/to/alacritty.yml.t] 
# the final name of parsed template, the same as defined above, but with the .t stripped
final_name = alacritty.yml
# lists are defined as a list of values separated by commas
hostnames = machine-one, machine-two, machine-three
# apply path, in other words, where should our output file be places
# if the path doesn't exits, der will attepmt to create it!
appy_path = /home/some/path/
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
$hostnames = terminator, cooldesk
$alacritty_out = ./out/alacritty/.config/alacritty/
$nvim_out = ./out/nvim/.config/nvim/
$i3_out = ./out/i3/.config/i3/
$dunst_out = .out/dunst/.config/dunst/
$bash_out = ./out/bash/

[alacritty/alacritty.yml.t]
final_name = alacritty.yml
hosntnames = $hostnames
apply_path = $alacritty_out

[nvim/init.lua.t]
final_name = init.lua
hostnames = $hostnames
apply_path = $nvim_out

# you can also copy entire folders like so:
# if no final_name field is provided, we use the same name
# if no hostnames filed is provided, we apply to all
[nvim/lua]
apply_path = $nvim_out

[i3/config.t]
final_name = config
hostnames = $hostnames
apply_path = $i3_out

[dunst/dunstrc.t]
final_name = dunstrc
honstames = $hostnames
apply_path = $dunst_out

[bash/bashrc.t]
final_name = .bashrc
hostnames = $hostnames
apply_path = $bash_out

[bash/tmux.conf.t]
final_name = .tmux.conf
hostnames = $hostnames
apply_path = $bash_out
```

Then, if we run the script from `dotfiles/`, we should have all templates applied and stowed to their exact locations.

## Template files
**TODO** template files are still not implemented.


# Contributing
Open an issue and if you feel like it, submit a PR.
