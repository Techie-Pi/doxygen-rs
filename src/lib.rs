#![allow(dead_code)]
#![allow(clippy::bool_assert_comparison)]

//! Simple Doxygen to Rustdoc translation.
//!
//! Provides a simple and straightforward API to translate _raw_ Doxygen comments to Rustdoc
//! comments. Purely experimental right now, maybe practical in a future?
//!
//! # Examples
//!
//! ```
//! use doxygen_rs::transform;
//!
//! let rustdoc = transform("@brief Example Doxygen brief");
//! assert_eq!(rustdoc, "Example Doxygen brief\n\n");
//! ```
//!
//! # Supported comments/notations
//! Currently the following notations are supported:
//! * [``brief``](https://www.doxygen.nl/manual/commands.html#cmdbrief)
//! * [``short``](https://www.doxygen.nl/manual/commands.html#cmdshort)
//! * [``param``](https://www.doxygen.nl/manual/commands.html#cmdparam)
//! * [``deprecated``](https://www.doxygen.nl/manual/commands.html#cmddeprecated)
//!
//! And the following _flavours_ are soported:
//! * ``\brief``
//! * ``\\brief``
//! * ``@brief``
//!
//! # Inner workings
//!
//! When the [``transform``] function is called, 3 other functions are called:
//! 1. The input is parsed to a [`Vec`] of [`parser::Value`] ([`parser::parse_comment`])
//! 2. The values are used to generate an AST ([`ast::generate_ast`])
//! 3. The AST is used to generate the Rustdoc ([`generator::generate_rustdoc`])
//!
//! ``transform [parse_comment -> generate_ast -> generate_rustdoc]``

use std::{fs, io};
use std::path::Path;
use crate::parser::StringType;

pub mod parser;
pub mod ast;
pub mod generator;
mod utils;

/// Transforms raw Doxygen comments to raw Rustdoc comments
///
/// # Examples
///
/// ```
/// use doxygen_rs::transform;
///
/// let rustdoc = transform("@brief Example Doxygen brief");
/// assert_eq!(rustdoc, "Example Doxygen brief\n\n");
/// ```
pub fn transform(input: &str) -> String {
    let parsed = parser::parse_comment(input);
    let ast = ast::generate_ast(parsed);
    generator::generate_rustdoc(ast)
}

pub fn transform_bindgen(input: &dyn AsRef<Path>) -> io::Result<String> {
    let mut file_data = vec![];
    let parsed = parser::parse_bindgen(fs::read_to_string(input)?.as_str());

    for parsed_data in parsed {
        println!("{:?}", parsed_data);
        match parsed_data {
            StringType::Parsed(data) => {
                let ast = ast::generate_ast(data);
                let rustdoc = generator::generate_rustdoc(ast);
                let bindgen_doc = rustdoc.lines().map(|v| format!("#[doc = \"{}\"]\n", v)).collect::<String>();
                file_data.push(bindgen_doc);
            },
            StringType::Raw(raw) => file_data.push(raw),
        }
    }

    Ok(file_data.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_transform() {
        println!("{}", transform("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name.\n@param[in] _test Test for In\n\n@deprecated\n\n@return a great thing"))
    }

    #[test]
    fn raw_transform_bindgen() {
        println!("{}", transform_bindgen(&"assets/tests/example-bindgen.rs").unwrap())
    }
}