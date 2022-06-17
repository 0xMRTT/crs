# C.R.S.

[![Rust](https://github.com/0xMRTT/crs/actions/workflows/rust.yml/badge.svg)](https://github.com/0xMRTT/crs/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/0xMRTT/crs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/0xMRTT/crs/actions/workflows/rust-clippy.yml)

Create a new project from a template

## Installation

You can download it on release page and simply run the binary

``` 
# NOT READY YET
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
