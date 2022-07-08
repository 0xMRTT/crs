# `CRSTemplate.json`

In this file, you can add hooks and metadata for the template.

This file is divised in twice part: `template` and `hooks`.

## `template`

In this part, you can add metadata.

Exemple:

```json
{
  "template": {
    "name": "rust-template",
    "version": "0.1.0",
    "description": "A template for Rust projects",
    "license": "MIT",
    "authors": ["0xMRTT <0xMRTT@tuta.io"],
    "maintainers": ["0xMRTT <0xMRTT@tuta.io"],
    "homepage": "https://github.com/0xMRTT/rust-template",
    "repository": "https://github.com/0xMRTT/rust-template",
    "documentation": "https://github.com/0xMRTT/rust-template",
    "keywords": ["rust", "template"]
  }
}
```

## `hooks`

In this part you can add hooks which will be run before and after the generation of the template.

Hooks are a dictionnary of key/value where key is the name of the hook and value is the function to run.
The value is an array of the command to run like `["/bin/sh", "-c" , "cargo build"]`.

Exemple:

```json
{
  "hooks": {
    "post": {
      "cargo build": ["/bin/sh", "-c", "cargo build"]
    }
  }
}
```
