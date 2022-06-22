# `crs.json`

`crs.json` file containing the data used in the template. The data is written in [JSON](https://en.wikipedia.org/wiki/JSON) format. 

There is one entry for one data. The entry is written in the following format:

``` json
{
    "email": {
        "type": "string",
        "default": "mail@example.com",
        "description": "Email of the author",
        "placeholder": "mail@example.com",
        "question": "What is the email of the author?",
        "validators": [
            "^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ],
        "error-message": "Email must be valid"
    }
}
```

## `type` (required)

Describe the type of the data. This can be :
- `"string"`
- `"boolean"`
- `"select"`
- `"multiselect"`

## `default` (optional)

The default value of the data.

## `description` (optional)

Description of the data used when `crs` ask the user.

## `placeholder` (optional)

Placeholder of the data used when `crs` ask the user.

## `question` (optional)

Question of the data used when `crs` ask the user.
If `question` isn't provided, `crs` will autogenerate a question.

## `validators` (optional)

An array of regexp used to validate the data. 

## `error-message` (optional)

Error message used when the data is invalid. (If `validators` is provided)
