# C.R.S.

There is an alternative repo on [codeberg](https://codeberg.org/0xMRTT/crs) : [https://codeberg.org/0xMRTT/crs](https://codeberg.org/0xMRTT/crs)

[![Rust](https://github.com/0xMRTT/crs/actions/workflows/rust.yml/badge.svg)](https://github.com/0xMRTT/crs/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/0xMRTT/crs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/0xMRTT/crs/actions/workflows/rust-clippy.yml)

Create a new project from a template

## Why another project generator ?

It's inspired of `cookiecutter` (#20). It's written in rust for safety and rapidity. CRS can run hooks before and after (#21) the generation. CRS use handelbars template language.

## Installation

### From crates.io

You can simply run `cargo install crs` and you'll have a `crs` command in your PATH.
After that, you can use `crs` to generate a project. 

``` shell
cargo install crs
crs --help
```

### As binary

You can download it on release page and simply run the binary

WARNING: not ready yet
``` 
wget https://github.com/0xMRTT/crs/
chmod +x crs
./crs
```

### From source

```
git clone https://github.com/0xMRTT/crs.git
cd crs
cargo b
```

And finally run crs

```
./target/debug/crs
```

## Usage

```
$ crs https://github.com/0xMRTT/rust-template

$ crs -h
crs 0.1.0

USAGE:
    crs [OPTIONS] [TEMPLATE_URL]

ARGS:
    <TEMPLATE_URL>    Optional name to operate on

OPTIONS:
    -c, --config <FILE>           Sets a custom config file
    -h, --help                    Print help information
    -l, --list-installed <DIR>    List installed template
    -t, --to <TO>                 Where CRS will generate the new project
    -V, --version                 Print version information
```

## Thanks

This project is inspired to this awesome projects:

* cookiecutter 

And thanks to the creators and contributors of this awesome rust crates:

* handlebars 
* serde_json 
* serde_derive 
* serde 
* env_logger 
* git2 
* clap 
* url
* walkdir
* platform-dirs 
* chrono
* inquire
* regex 
* execute 
* fs_extra 

## Community

* [Discord](https://discord.gg/Umnpj9vnjR)
* [Matrix]()

## We're Using GitHub Under Protest

This project is currently hosted on GitHub.  This is not ideal; GitHub is a proprietary, trade-secret system that is not Free and Open Souce Software (FOSS).  We 
are deeply concerned about using a proprietary system like GitHub to develop our FOSS project.  We have an [open {bug ticket, mailing list thread, etc.} ](INSERT_LINK)
where the project contributors are actively discussing how we can move away from GitHub in the long term.  We urge you to read about the [Give up GitHub](https://GiveUpGitHub.org) campaign from [the Software Freedom Conservancy](https://sfconservancy.org) to understand some of the reasons why GitHub is not
a good place to host FOSS projects.

If you are a contributor who personally has already quit using GitHub, please [check this resource](INSERT_LINK) for how to send us contributions without using
GitHub directly.

![Logo of the GiveUpGitHub campaign](https://sfconservancy.org/img/GiveUpGitHub.png)
