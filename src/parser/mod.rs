//! This module contains functions to generate a [`Vec`] of [`Value`] from a given input
//!
//! **The functions and structs here should _not_ be considered stable**

use crate::utils::NotationMatching;

/// The enum used to represent the distinct _raw_ values of a comment
#[derive(Debug, Clone)]
pub enum Value {
    /// The first [`String`] is the _notation_ found, and the second [`String`] are the _contents without the notation_
    Notation(String, String),
    /// Raw text, without notation
    Text(String),
    /// Double new-line, or any other separator
    Separator,
    /// Unknown value
    Unknown,
}

/// Generate a [`Vec`] of [`Value`] from a given [`&str`]
///
/// # Examples
/// ```
/// use doxygen_rs::parser::parse_comment;
///
/// let parsed = parse_comment("@brief Random function");
/// ```
pub fn parse_comment(input: &str) -> Vec<Value> {
    let lines = input.split('\n').map(|v| v.to_string()).collect::<Vec<String>>();
    let mut ast = vec![];

    for line in lines {
        if let Some(notation) = line.contains_any_notation() {
            let split = line.split_whitespace().collect::<Vec<&str>>();
            ast.push(Value::Notation(notation.clone(), split[1..].to_vec().join(" ").to_string()));
        } else if line.is_empty() {
            ast.push(Value::Separator);
        } else {
            ast.push(Value::Text(line));
        }
    }
    ast.push(Value::Separator);

    ast
}

#[derive(Clone, Debug)]
pub enum StringType {
    Parsed(Vec<Value>),
    Raw(String),
}

pub fn parse_bindgen(input: &str) -> Vec<StringType> {
    let lines: Vec<String> = input.split('\n').map(|v| v.to_string()).collect::<Vec<String>>();
    let mut strings = vec![];

    let mut comment_buffer = vec![];
    for line in lines {
        if line.trim().starts_with("#[doc = \"") && line.trim().ends_with("\"]") {
            comment_buffer.push(line.replace("#[doc = \"", "").replace("\"]", ""));
        } else {
            if !comment_buffer.is_empty() {
                strings.push(StringType::Parsed(parse_comment(comment_buffer.join("\n").as_str())));
                comment_buffer = vec![];
            }
            strings.push(StringType::Raw(line));
        }
    }

    strings
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_comment;

    #[test]
    fn test() {
        let ast = parse_comment("@param random Random thing lmao\n\n@block This is going to be\nA block of text\nThis is crazy right??\n\nHello this is not anotated\n");
        println!("{:?}", ast);
    }
}

