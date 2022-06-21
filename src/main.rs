#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use git2::Repository;
use handlebars::{
    to_json, Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};
use serde::Serialize;
use serde_json::value::{self, Map, Value as Json};
use serde_json::{json, Number, Value};
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

use chrono::Datelike;
use execute::Execute;
use inquire::error::InquireError;
use inquire::*;
use platform_dirs::{AppDirs, UserDirs};
use regex::Regex;
use std::process::exit;
use std::process::{Command, Stdio};
use walkdir::WalkDir;

extern crate fs_extra;
use fs_extra::dir::copy;
use fs_extra::dir::CopyOptions;

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
    let current_date = chrono::Utc::now();
    data.insert("year".to_string(), to_json(current_date.year()));
    data.insert("month".to_string(), to_json(current_date.month()));
    data.insert("day".to_string(), to_json(current_date.day()));

    let mut crs_data = Map::new();
    crs_data.insert("engine".to_string(), to_json(TYPES));

    let mut template_data = Map::new();
    template_data.insert("name".to_string(), to_json(template_name));
    template_data.insert("url".to_string(), to_json(template_url));
    template_data.insert("author".to_string(), to_json(template_author));
    template_data.insert("username".to_string(), to_json(template_username));

    crs_data.insert("template".to_string(), to_json(template_data));

    data.insert("crs".to_string(), to_json(crs_data));

    data.insert("d".to_string(), to_json(ask_user(json_data_path)));
    data
}

fn get_user_default() -> serde_json::Map<std::string::String, Value> {
    let app_dirs = AppDirs::new(Some("crs"), false).unwrap();
    let user_defaults = app_dirs.config_dir.join("defaults.json");
    let json_data = {
        // Load the first file into a string.
        let text = std::fs::read_to_string(user_defaults).unwrap();

        // Parse the string into a dynamically-typed JSON structure.
        serde_json::from_str::<Value>(&text).unwrap()
    };
    let mut data = Map::new();
    for (key, value) in json_data.as_object().unwrap().iter() {
        data.insert(key.to_string(), to_json(value.as_str().unwrap()));
    }
    return data;
}

fn generate_file(
    handlebars: &mut handlebars::Handlebars,
    template: &str,
    output_file: &str,
    data: &Map<String, Json>,
) -> Result<(), Box<dyn Error>> {
    handlebars.register_template_string(template, fs::read_to_string(template)?)?;
    //handlebars.register_template_file(template, template).expect("Failed to register template");
    let mut output_file = File::create(output_file)?;
    handlebars.render_to_write(template, &data, &mut output_file)?;
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

    /// Where CRS will generate the new project
    #[clap(short, long)]
    to: Option<String>,

    /// List installed template
    #[clap(short, long, value_name = "DIR")]
    list_installed: bool,
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
    fs::create_dir_all(&to.clone()).unwrap();

    let paths = fs::read_dir(folder_path.as_str()).unwrap();
    let folder_content = paths.map(|path| path.unwrap().path());
    for path in folder_content {
        let file_name = path.file_name().unwrap().to_str().unwrap();

        let mut new: String = to.clone();
        new.push('/');
        new.push_str(file_name);
        new = generate_name(handlebars, &new, data);
        if path.is_dir() {
            if path.display().to_string().contains(".git") {
                continue;
            }
            let new_folder_path = folder_path.clone() + "/" + file_name;
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

fn validate(regexp: &str, value: &str) -> bool {
    let re = Regex::new(regexp).unwrap();
    re.is_match(value)
}

fn ask_user(
    template_json_path: String,
) -> serde_json::Map<std::string::String, handlebars::JsonValue> {
    let json_data = {
        // Load the first file into a string.
        let text = std::fs::read_to_string(&template_json_path).unwrap();

        // Parse the string into a dynamically-typed JSON structure.
        serde_json::from_str::<Value>(&text).unwrap()
    };
    let mut data = Map::new();
    let defaults = get_user_default(); // Load the user defaults from ~/.config/crs/defaults.json

    for (key, value) in json_data.as_object().unwrap().iter() {
        let default_value = value.get("default");
        let mut default = ""; // "" is the default value
        if default_value != None {
            // use default value provided by the creator of the template in 'crs.json'
            default = default_value.unwrap().as_str().unwrap();
        }
        if defaults.contains_key(key) {
            // use default value provided by the user
            default = defaults.get(key).unwrap().as_str().unwrap();
        }

        let description_value = value.get("description");
        let mut description = ""; // "" is the default value
        if description_value != None {
            // use default value provided by the creator of the template in 'crs.json'
            description = description_value.unwrap().as_str().unwrap();
        }

        let placeholder_value = value.get("placeholder");
        let mut placeholder = ""; // "" is the default value
        if placeholder_value != None {
            // use default value provided by the creator of the template in 'crs.json'
            placeholder = placeholder_value.unwrap().as_str().unwrap();
        }

        let question_value = value.get("question");
        let mut question = format!("What is {} ?", key); // Default question
        if question_value != None {
            // use default value provided by the creator of the template in 'crs.json'
            question = question_value.unwrap().as_str().unwrap().to_string();
        }

        let validators = value.get("validators");
        let mut _validators_list = Vec::new();
        let mut validators_list = Vec::new();
        let mut is_validators = false;
        if validators != None {
            // use default value provided by the creator of the template in 'crs.json'
            _validators_list = validators.unwrap().as_array().unwrap().to_vec();
            for validator in _validators_list.iter() {
                validators_list.push(validator.as_str().unwrap());
            }
            is_validators = true;
        }

        let error_message_value = value.get("error-message");
        let mut error_message = validators_list.join(", "); // Default error message
        if error_message_value != None {
            // use default value provided by the creator of the template in 'crs.json'
            error_message = error_message_value.unwrap().as_str().unwrap().to_string();
        }

        let mut is_value_correct = false;

        while !is_value_correct {
            if value["type"] == "select" {
                let choices = value["options"].as_array().unwrap().to_vec();
                let options = choices
                    .iter()
                    .map(|choice| choice.as_str().unwrap())
                    .collect();
                let result: Result<&str, InquireError> = Select::new(question.as_str(), options)
                    .with_help_message(description)
                    .prompt();

                let r = result.unwrap();
                if is_validators {
                    for validator in &validators_list {
                        if validate(validator, &r) != true {
                            println!("{} is not valid. {}", &r, error_message);
                        } else {
                            is_value_correct = true;
                        }
                    }
                } else {
                    is_value_correct = true;
                }
                data.insert(key.to_string(), Json::String(r.to_string()));
            } else if value["type"] == "multiselect" {
                let choices = value["options"].as_array().unwrap().to_vec();
                let options = choices
                    .iter()
                    .map(|choice| choice.as_str().unwrap())
                    .collect();
                let result = MultiSelect::new(question.as_str(), options)
                    .with_help_message(description)
                    .prompt();

                let r = result.unwrap();
                if is_validators {
                    for validator in &validators_list {
                        for r_ in r.iter() {
                            if validate(validator, &r_) != true {
                                println!("{} is not valid. {}", &r_, error_message);
                            } else {
                                is_value_correct = true;
                            }
                        }
                    }
                } else {
                    is_value_correct = true;
                }
                data.insert(key.to_string(), to_json(r));
            } else if value["type"] == "boolean" {
                let result = Confirm::new(question.as_str())
                    .with_help_message(description)
                    .with_default(default.parse::<bool>().unwrap())
                    .prompt();
                data.insert(key.to_string(), to_json(result.unwrap()));
            } else {
                // by default, it's string even if the type isn't specified
                let result = Text::new(question.as_str())
                    .with_placeholder(placeholder)
                    .with_default(default)
                    .with_help_message(description)
                    .prompt();

                let r = result.unwrap();
                if is_validators {
                    for validator in &validators_list {
                        if validate(&validator, &r.as_str()) != true {
                            println!("{} is not valid. {}", &r.as_str(), error_message);
                        } else {
                            is_value_correct = true;
                        }
                    }
                } else {
                    is_value_correct = true;
                }
                data.insert(key.to_string(), Json::String(r));
            }
        }
    }

    return to_json(data).as_object().unwrap().clone();
}

fn run_hooks(clone_to: PathBuf) {
    run_post_hooks(clone_to)
}

fn run_post_hooks(clone_to: PathBuf) {
    println!("Running post hooks");
    println!("clone_to: {}", clone_to.display());
    let mut crs_template_json_path = clone_to.clone();
    crs_template_json_path.push("CRSTemplate.json");
    println!("{}", crs_template_json_path.to_str().unwrap());

    let crs_template_json = {
        // Load the first file into a string.
        let text = std::fs::read_to_string(crs_template_json_path).unwrap();

        // Parse the string into a dynamically-typed JSON structure.
        serde_json::from_str::<Value>(&text).unwrap()
    };

    let hooks = crs_template_json["hooks"].as_object().unwrap();
    let post_hooks = hooks["post"].as_object().unwrap();
    for (key, value) in post_hooks.iter() {
        println!("Running post hook {}", key);
        let _command_vec = value.as_array().unwrap().to_vec();
        let mut command_vec = _command_vec;

        let command_str = command_vec[0].as_str().unwrap().to_string();
        command_vec.remove(0);
        let mut _args = command_vec
            .iter()
            .map(|arg| arg.as_str().unwrap())
            .collect::<Vec<&str>>();

        println!("command: {}", command_str);
        println!("args: {:?}", _args);

        let mut child = Command::new(command_str)
            .args(_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute child");

        // If the child process fills its stdout buffer, it may end up
        // waiting until the parent reads the stdout, and not be able to
        // read stdin in the meantime, causing a deadlock.
        // Writing from another thread ensures that stdout is being read
        // at the same time, avoiding the problem.
        let mut stdin = child.stdin.take().expect("failed to get stdin");
        std::thread::spawn(move || {
            stdin.write_all(b"test").expect("failed to write to stdin");
        });

        let output = child.wait_with_output().expect("failed to wait on child");

        println!("Result of hook: {:?}", output.stdout.as_slice());
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let mut to: String = "generated".to_string();
    if cli.to.is_some() {
        to = cli.to.unwrap();
    }

    if cli.list_installed {
        list_installed();
        exit(0);
    } else if cli.template_url.is_some() {
        let template_url = cli.template_url.unwrap();

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

        let current_dir = std::env::current_dir()?;

        println!("Thanks to @{} for creating {}. You can create your own template. RTD for more (https://0xMRTT.github.io/docs/crs)", username.unwrap(), template_name.unwrap());
        if clone_to.exists() {
            env::set_current_dir(template_store_path)?;
            let redownload = Confirm::new(
                "Template already downloaded. Do you want to re-download the template ?",
            )
            .with_default(true)
            .prompt();

            match redownload {
                Ok(true) => {
                    let to_delete = template_name.unwrap();
                    let path_to_delete = Path::new(&to_delete);
                    println!("Deleting old template ({})", &to_delete);
                    fs::remove_dir_all(path_to_delete)?;
                    println!("Clone {} to {:#?}", template_url, clone_to);
                    clone_repo(template_url, &clone_to).expect("");
                    println!("Successfuly downloaded template.");
                }
                Ok(false) => {
                    let sure = Confirm::new("Are you sure ?").with_default(false).prompt();

                    match sure {
                        Ok(true) => println!("Skip re-downloading of the template."),
                        Ok(false) => {
                            let to_delete = template_name.unwrap();
                            let path_to_delete = Path::new(&to_delete);
                            println!("Deleting old template ({})", &to_delete);
                            fs::remove_dir_all(path_to_delete)?;
                            println!("Clone {} to {:#?}", template_url, clone_to);
                            clone_repo(template_url, &clone_to).expect("");
                            println!("Successfuly downloaded template.");
                        }
                        Err(_) => println!("Error, try again later"),
                    }
                }
                Err(_) => println!("Error, try again later"),
            }
            env::set_current_dir(current_dir)?; // Come back to the current directory
        } else {
            println!("Clone {} to {:#?}", template_url, clone_to);

            clone_repo(template_url, &clone_to).expect("");

            println!("Successfuly downloaded template.");
        }

        let mut temp_dir = env::temp_dir();

        let options = CopyOptions::new();

        temp_dir.push("crs");

        fs::create_dir_all(temp_dir.clone())?;

        copy(clone_to.clone(), temp_dir.clone(), &options)?;

        temp_dir.push(template_name.unwrap());

        println!("Copy template to {} (temp)", temp_dir.display());

        let mut folder_path = temp_dir.clone().to_str().unwrap().to_string();
        folder_path.push_str("/template");

        let mut _json_data_file = String::new();
        if cli.config.is_some() {
            _json_data_file = cli.config.unwrap().display().to_string();
        } else {
            _json_data_file = temp_dir.to_str().unwrap().to_string() + "/crs.json";
        }

        let mut handlebars = Handlebars::new();

        handlebars.register_helper("format", Box::new(format_helper));
        handlebars.register_helper("ranking_label", Box::new(rank_helper));

        let data = make_data(
            "basic".to_string(),
            "https://github.com/0xMRTT/basic-template".to_string(),
            "0xMRTT".to_string(),
            "0xMRTT".to_string(),
            _json_data_file.clone(),
        );

        if to != "generated" {
            to = generate_name(&mut handlebars, &to, &data);
        }
        println!("Generating project to {} from {}", to, folder_path);
        generate_folder(&mut handlebars, &folder_path, &to, &data);
        println!("Project generated. Happy coding!");
        env::set_current_dir(to)?;
        println!("Run hooks...");
        run_hooks(clone_to);
        println!("Deleting temp dir");
        fs::remove_dir_all(temp_dir.clone())?;
    } else {
        println!("No template url provided. Use --help for more information.");
    }
    Ok(())
}
