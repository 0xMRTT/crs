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

use git2::Repository;
use handlebars::{
    to_json, Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path;
use std::path::Path;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use url::{ParseError, Url};

use platform_dirs::{AppDirs, UserDirs};
use std::process::exit;
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

static TYPES: &str = "serde_json";

pub fn make_data(
    template_name: String,
    template_url: String,
    template_author: String,
    template_username: String,
    json_data_path: String,
) -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json("2022"));

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
    data
}

fn generate_file(
    handlebars: &mut handlebars::Handlebars,
    template: &str,
    output_file: &str,
    data: &Map<String, Json>,
) -> Result<(), Box<dyn Error>> {
    handlebars.register_template_string(template, fs::read_to_string(template)?)?;
    //handlebars.register_template_file(template, template).expect("Failed to register template");
    let output_file_path = output_file;
    let mut output_file = File::create(output_file)?;
    handlebars.render_to_write(template, &data, &mut output_file)?;
    println!(" |--> {} generated", output_file_path);
    Ok(())
}

fn clone_repo(url: String, to: &std::path::PathBuf) -> Result<git2::Repository, Box<dyn Error>> {
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
    template_url: Option<String>,

    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,

    #[clap(short, long)]
    to: Option<String>,

    #[clap(short, long)]
    list_installed: bool,

    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    json_data_path: Option<PathBuf>,
}

fn list_installed() {
    let app_dirs = AppDirs::new(Some("crs"), false).unwrap();
    let template_store_path = &app_dirs.data_dir.clone();

    let paths = fs::read_dir(template_store_path).unwrap();

    let mut number = 0;
    println!("Installed templates:");
    for path in paths {
        println!(" - {}", path.unwrap().path().display());
        number += 1;
    }
    println!("{} templates installed", number);
}

fn generate_name(
    handlebars: &mut handlebars::Handlebars,
    original_name: &String,
    data: &Map<String, Json>,
) -> String {
    // Generate the name of the template
    //
    // Example:
    //   println!("{}", generate_name(&mut handlebars, "{{d.project_name}}.md".to_string(), data));

    return handlebars
        .render_template(original_name.as_str(), &data)
        .unwrap();
}

fn generate_folder(
    handlebars: &mut handlebars::Handlebars,
    folder_path: &String,
    to: &String,
    data: &Map<String, Json>,
) {
    println!("Generating project to {} from {}", to, folder_path);

    fs::create_dir_all(&to.clone()).unwrap();

    let paths = fs::read_dir(folder_path.as_str()).unwrap();
    let folder_content = paths.map(|path| path.unwrap().path());
    for path in folder_content {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        println!(" - {}", file_name);
        println!(" |- {}", path.display());

        let mut new: String = to.clone();
        new.push_str("/");
        new.push_str(file_name);
        new = generate_name(handlebars, &new, data);
        println!(" |--> {}", new);
        if path.is_dir() {
            if path.display().to_string().contains(".git") {
                println!(" |--> Skipping");
                continue;
            }
            let new_folder_path = folder_path.clone() + "/" + file_name;
            println!(" |---> {}", new_folder_path);
            generate_folder(handlebars, &new_folder_path, &new, data);
        } else {
            generate_file(
                handlebars,
                path.display().to_string().as_str(),
                new.as_str(),
                data,
            )
            .unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    
    let cli = Cli::parse();
    
    let mut to:String = "generated".to_string();
    
    if cli.to.is_some() {
        to = cli.to.unwrap();
    }

    if cli.list_installed {
        list_installed();
        exit(0);
    } else if cli.template_url.is_some() && cli.json_data_path.is_some() {
        let template_url = cli.template_url.unwrap();
        let json_data_file = cli.json_data_path.unwrap();
        println!("Generating a new project using {}", template_url);

        let app_dirs = AppDirs::new(Some("crs"), false).unwrap();
        let template_store_path = &app_dirs.data_dir.clone();

        println!(
            "Creating store directory in {}",
            template_store_path.to_str().unwrap()
        );
        fs::create_dir_all(&app_dirs.data_dir).unwrap();

        let url = Url::parse(template_url.as_str())?;
        let mut path_segments = url.path_segments().ok_or_else(|| "cannot be base")?;

        let username = path_segments.next();
        let template_name = path_segments.next();

        let mut clone_to = app_dirs.data_dir;
        clone_to.push(template_name.unwrap());

        println!("Thanks to {} for creating {}. You can create your own template. RTD for more (https://0xMRTT.github.io/docs/crs)", username.unwrap(), template_name.unwrap());
        if clone_to.exists() {
            println!("Template already downloaded. Updating...");
            env::set_current_dir(template_store_path)?;
            let to_delete = template_name.unwrap();
            let path_to_delete = Path::new(&to_delete);
            println!("Deleting old template ({})", &to_delete);
            fs::remove_dir_all(path_to_delete)?;
        }
        println!("Clone {} to {:#?}", template_url, clone_to);

        clone_repo(template_url, &clone_to).expect("");

        println!("Successfuly downloaded template.");

        println!("Start generating new project...");

        println!("WIP");

        let data = make_data(
            "basic".to_string(),
            "https://github.com/0xMRTT/basic-template".to_string(),
            "0xMRTT".to_string(),
            "0xMRTT".to_string(),
            json_data_file.display().to_string(),
        );

        println!("Using data from {:#?}", json_data_file.display());
        // START : Create global handelbars
        let mut handlebars = Handlebars::new();

        handlebars.register_helper("format", Box::new(format_helper));
        handlebars.register_helper("ranking_label", Box::new(rank_helper));
        // handlebars.register_helper("format", Box::new(FORMAT_HELPER));

        // END: Create global handelbars

        /*generate_file(
            &mut handlebars,
            "~/Projects/crs/src/template.hbs",
            "README.md",
            data,
        )?;*/
        let folder_path = clone_to.display().to_string() + "/template";
        
        if to != "generated".to_string() {
            to = generate_name(&mut handlebars, &to, &data);
        }
        generate_folder(
            &mut handlebars,
            &folder_path,
            &to,
            &data,
        );
    } else {
        println!("https://github.com/0xMRTT/basic-template");
    }
    Ok(())
}
