#![allow(unused_imports, dead_code)]
extern crate env_logger;
extern crate handlebars;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde::Serialize;
use serde_json::value::{self, Map, Value as Json};
use serde_json::{Value, Number};

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use handlebars::{
    to_json, Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};

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
pub fn make_data(template_name:String, template_url:String, template_author:String, template_username:String, json_data_path:String) -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("year".to_string(), to_json("2022"));
    data.insert("project_name".to_string(), to_json("C.R.S."));
    data.insert("description".to_string(), to_json("Project generator with rust"));

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

fn generate_file(handlebars: &mut handlebars::Handlebars, template:&str, output_file:&str, json_data_file:&str) -> Result<(), Box<dyn Error>> {

    let data = make_data("basic".to_string(), "https://github.com/0xMRTT/basic-template".to_string(), "0xMRTT".to_string(), "0xMRTT".to_string(), json_data_file.to_string());

    handlebars
        .register_template_file("template", template)
        .unwrap();

    let output_file_path = output_file;
    let mut output_file = File::create(output_file)?;
    handlebars.render_to_write("template", &data, &mut output_file)?;
    println!("{} generated", output_file_path);
    Ok(())

}


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut handlebars = Handlebars::new();

    handlebars.register_helper("format", Box::new(format_helper));
    handlebars.register_helper("ranking_label", Box::new(rank_helper));
    // handlebars.register_helper("format", Box::new(FORMAT_HELPER));


    generate_file(&mut handlebars, "./src/template.hbs", "target/README.md", "./src/data.json")?;
    Ok(())
}