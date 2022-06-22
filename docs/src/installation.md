# Installation

## From [crates.io](https://crates.io)

You can simply run `cargo install crs` and you'll have a `crs` command in your PATH.
After that, you can use `crs` to generate a project. 

``` shell
cargo install crs
crs --help
```

## As binary

You can download it on release page and simply run the binary

WARNING: not ready yet
``` 
wget https://github.com/0xMRTT/crs/
chmod +x crs
./crs
```

## From source

```
git clone https://github.com/0xMRTT/crs.git
cd crs
cargo b
```

And finally run crs

```
./target/debug/crs
```