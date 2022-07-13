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

mod parser;
mod ast;
mod generator;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_transform() {
        println!("{}", transform("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name.\n@param[in] _test Test for In\n\n@deprecated"))
    }
}