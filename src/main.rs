#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde::Serialize;
use serde_json::value::{self, Map, Value as Json};
use serde_json::{Number, Value};

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use git2::Repository;

use handlebars::{
    to_json, Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use url::{ParseError, Url};

use walkdir::WalkDir;

// define a custom helper
fn format_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let param = h
        .param(0)
        .ok_or(RenderError::new("Param 0 is required for format helper."))?;
    let rendered = format!("{} pts", param.value().render());
    out.write(rendered.as_ref())?;
    Ok(())
}

// another custom helper
fn rank_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let rank = h
        .param(0)
        .and_then(|ref v| v.value().as_u64())
        .ok_or(RenderError::new(
            "Param 0 with u64 type is required for rank helper.",
        ))? as usize;
    let total = h
        .param(1)
        .as_ref()
        .and_then(|v| v.value().as_array())
        .map(|arr| arr.len())
        .ok_or(RenderError::new(
            "Param 1 with array type is required for rank helper",
        ))?;
    if rank == 0 {
        out.write("champion")?;
    } else if rank >= total - 2 {
        out.write("relegation")?;
    } else if rank <= 2 {
        out.write("acl")?;
    }
    Ok(())
}

static TYPES: &'static str = "serde_json";

// define some data
#[derive(Serialize)]
pub struct Team {
    name: String,
    pts: u16,
}

// produce some data
pub fn make_data(
    template_name: String,
    template_url: String,
    template_author: String,
    template_username: String,
    json_data_path: String,
) -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json("2022"));
    data.insert("project_name".to_string(), to_json("C.R.S."));
    data.insert(
        "description".to_string(),
        to_json("Project generator with rust"),
    );

    let mut crs_data = Map::new();
    crs_data.insert("engine".to_string(), to_json(TYPES));

    let mut template_data = Map::new();
    template_data.insert("name".to_string(), to_json(template_name));
    template_data.insert("url".to_string(), to_json(template_url));
    template_data.insert("author".to_string(), to_json(template_author));
    template_data.insert("username".to_string(), to_json(template_username));

    crs_data.insert("template".to_string(), to_json(template_data));

    data.insert("crs".to_string(), to_json(crs_data));

    let json_data = {
        // Load the first file into a string.
        let text = std::fs::read_to_string(json_data_path).unwrap();

        // Parse the string into a dynamically-typed JSON structure.
        serde_json::from_str::<Value>(&text).unwrap()
    };

    data.insert("d".to_string(), json_data);
    println!("{:#?}", data);
    data
}

fn generate_file(
    handlebars: &mut handlebars::Handlebars,
    template: &str,
    output_file: &str,
    json_data_file: &str,
) -> Result<(), Box<dyn Error>> {
    let data = make_data(
        "basic".to_string(),
        "https://github.com/0xMRTT/basic-template".to_string(),
        "0xMRTT".to_string(),
        "0xMRTT".to_string(),
        json_data_file.to_string(),
    );

    handlebars
        .register_template_file("template", template)
        .unwrap();

    let output_file_path = output_file;
    let mut output_file = File::create(output_file)?;
    handlebars.render_to_write("template", &data, &mut output_file)?;
    println!("{} generated", output_file_path);
    Ok(())
}

fn clone_repo(url: String, to: String) -> Result<git2::Repository, Box<dyn Error>> {
    let final_path = to;
    let _repo = match Repository::clone(&url, final_path) {
        Ok(repo) => return Ok(repo),
        Err(e) => panic!("failed to clone: {}", e),
    };
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    template_url: String,

    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,

    #[clap(short, long)]
    to: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let template_url = cli.template_url;

    let to_path: String;

    if let Some(to) = cli.to.as_deref() {
        to_path = to.to_string();
    } else {
        let url_parsed = Url::parse(&template_url)?;
        let mut path_segments = url_parsed.path_segments().ok_or_else(|| "cannot be base")?;
        _ = path_segments.next(); // username
        to_path = path_segments.next().unwrap().to_string();
    }
    let path = Path::new(&to_path);

    clone_repo(template_url, to_path.to_string())?;

    println!("PATH : {}", to_path);
    std::env::set_current_dir(path)?;
    println!("DIRR : -> {:#?}", std::env::current_dir());

    // START : Create global handelbars
    let mut handlebars = Handlebars::new();

    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    // handlebars.register_helper("format", Box::new(FORMAT_HELPER));

    // END: Create global handelbars

    for entry in WalkDir::new("foo") {
        println!("{}", entry?.path().display());
    }

    generate_file(
        &mut handlebars,
        "./src/template.hbs",
        "target/README.md",
        "./src/data.json",
    )?;
    Ok(())
}
