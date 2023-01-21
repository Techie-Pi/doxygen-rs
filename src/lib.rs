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
//! # Usage with bindgen >= 0.63
//!
//! ```compile_fail
//! #[derive(Debug)]
//! struct Cb;
//!
//! impl ParseCallbacks for Cb {
//!     fn process_comment(&self, comment: &str) -> Option<String> {
//!         Some(doxygen_rs::transform(comment))
//!     }
//! }
//! ```
//!
//! # Supported commands
//! See the [tracking issue](https://github.com/Techie-Pi/doxygen-rs/issues/1) for the exhaustive list
use crate::parser::StringType;

pub mod ast;
pub mod generator;
pub mod parser;
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

pub fn transform_bindgen(input: &str) -> String {
    let mut file_data = vec![];
    let parsed = parser::parse_bindgen(input);

    for parsed_data in parsed {
        match parsed_data {
            StringType::Parsed(data) => {
                let ast = ast::generate_ast(data);
                let rustdoc = generator::generate_rustdoc(ast);
                let bindgen_doc = rustdoc
                    .lines()
                    .map(|v| format!("#[doc = \"{}\"]\n", v.trim()))
                    .collect::<String>();
                file_data.push(bindgen_doc);
            }
            StringType::Raw(raw) => file_data.push(raw),
        }
    }

    file_data.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn raw_transform() {
        const INPUT: &str = r#"
@brief Creates a new dog.

Creates a new Dog named `_name` with half
    of its maximum
    energy.


@param _name The dog's name. Ignored when:
    - _test input null
    - random_dog cfg option set 
@param[in] _test Test for In,
    also testing longer param description
    - also param sublist
      - and nested sublist

@deprecated

@return a great thing
"#;

        const EXPECTED: &str = r#"**Warning!** This is deprecated!

Creates a new dog.

Creates a new Dog named `_name` with half of its maximum energy.

Returns:

* a great thing

# Arguments

* `_name` - The dog's name. Ignored when:
  * _test input null
  * random_dog cfg option set 
* `_test` - [Direction: In] Test for In, also testing longer param description
  * also param sublist
    * and nested sublist

"#;
        assert_eq!(EXPECTED, transform(INPUT).as_str(),);
    }

    #[test]
    fn raw_transform_bindgen() {
        let file = fs::read_to_string("assets/tests/example-bindgen.rs").unwrap();
        let result = transform_bindgen(file.as_str());
        let _ = fs::remove_file("assets/tests/bindgen-transformed.rs");
        fs::write("assets/tests/bindgen-transformed.rs", result).unwrap();
    }

    #[test]
    fn transform_ctru_sys_bindings() {
        let file = fs::read_to_string("assets/tests/ctru-sys-bindings.rs").unwrap();
        let result = transform_bindgen(file.as_str());
        let _ = fs::remove_file("assets/tests/ctru-sys-bindings-transformed.rs");
        fs::write("assets/tests/ctru-sys-bindings-transformed.rs", result).unwrap();
    }
}
