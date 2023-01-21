//! This module contains functions to generate a [`Vec`] of [`Value`] from a given input
//!
//! **The functions and structs here should _not_ be considered stable**

use std::fmt::Display;

use crate::utils::NotationMatching;

mod preprocessor;
#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Default)]
pub struct NestedString {
    pub top: String,
    pub sub: Vec<NestedString>,
}


impl NestedString {
    pub fn new(top: String) -> Self {
        Self { top, sub: vec![] }
    }

    pub fn modify_sub(mut self, nesting_depth: usize, sublist: bool, line: String) -> NestedString {
        let mut refs_vec = vec![self];
        for _ in 1..nesting_depth {
            let mut high = refs_vec.pop().unwrap(); // Unwrap won't panic, because `refs_vec` has initial len > 0
            let low = high.sub.pop();
            refs_vec.push(high);
            if let Some(low) = low {
                refs_vec.push(low);
            } else {
                refs_vec.push(NestedString::default());
            }
        }
        if sublist {
            refs_vec
                .last_mut()
                .unwrap()
                .sub
                .push(NestedString::new(line));
        } else {
            refs_vec.last_mut().unwrap().top.push(' ');
            refs_vec.last_mut().unwrap().top.push_str(line.as_str());
        }

        for _ in 1..nesting_depth {
            let last = refs_vec.pop().unwrap();
            refs_vec.last_mut().unwrap().sub.push(last);
        }
        self = refs_vec.swap_remove(0);
        self
    }
}
impl Display for NestedString {
    /// usable formating options:
    /// :{width} - NestedString.top intendation/sublist depth, NestedString.sub is one sublist level deeper
    /// :# - NestedString.top is at 0 sublist depth(no sublist at all), but NestedString.sub is on sublist level width+1
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line_prefix = String::new();
        let width = formatter.width().unwrap_or(0);
        if !(width == 0 || formatter.alternate()) {
            line_prefix += format!("{:>width$}", "* ", width = 2 * width).as_str();
            // Set element's initial depth in list
        }

        {
            write!(formatter, "{}{}", line_prefix, self.top)?;
        }

        if !self.sub.is_empty() {
            for ns in self.sub.iter() {
                write!(formatter, "\n{:width$}", ns, width = width + 1)?;
            }
        }
        write!(formatter, "")
    }
}

/// The enum used to represent the distinct _raw_ values of a comment
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    /// The first [`String`] is the _notation_ found, and .top of [`NestedString`] are the _contents without the notation_; the .sub of [`NestedString`] is a vector that contains consecutive list (lines starting with "-")
    Notation(String, NestedString),
    /// Raw text, without notation
    Text(NestedString),
    /// Double new-line, or any other separator
    Separator,
    /// indented Text- probably continuation of previous line; [`String`]- line text stripped of leading whitespaces & sublist characters as '-','*'and '+'; [`usize`] - number of leading whitespaces; [`bool`]- sublist char present?
    Continuation(String, usize, bool),
    /// Unknown value
    Unknown,
}

fn parse_single_line(line: &str) -> Value {
    let mut line = preprocessor::preprocess_line(line);
    let mut leading_whitespaces = 0;
    let mut chars_to_skip = 0;
    let mut sublist = false;
    for ch in line.chars() {
        if ch.is_whitespace() {
            leading_whitespaces += 1;
        } else if ch == '-' || ch == '*' || ch == '+' {
            sublist = true;
            chars_to_skip = leading_whitespaces + 1;
            break;
        } else {
            chars_to_skip = leading_whitespaces;
            break;
        };
    }
    line.drain(..chars_to_skip); // Remove leading whitespaces and sublist mark
    line = line.trim_start().to_string(); // Remove leading whitespaces after sublist mark
    if let Some(notation) = line.contains_any_notation() {
        let split = line.split_whitespace().collect::<Vec<&str>>();
        Value::Notation(notation, NestedString::new(split[1..].to_vec().join(" ")))
    } else if line.is_empty() {
        Value::Separator
    } else if leading_whitespaces > 0 {
        Value::Continuation(line, leading_whitespaces, sublist)
    } else {
        Value::Text(NestedString::new(line))
    }
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
    let lines = input
        .split('\n')
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    let mut values = vec![];
    let mut nesting_depth = 0;
    let mut nesting_spaces = vec![0];

    for line in lines {
        let value = if line.trim().starts_with("* ") {
            parse_single_line(line.replace("* ", "").as_str())
        } else {
            parse_single_line(line.as_str())
        };
        if let Value::Continuation(line, leading_whitespaces, sublist) = value {
            for nd in (0..=nesting_depth).rev() {
                // Assume that sublist level takes at least two whitespaces, and single whitespace is a typo
                if leading_whitespaces >= nesting_spaces[nd] + 2 {
                    // Sublist level has increased
                    nesting_depth += 1;
                    nesting_spaces.push(leading_whitespaces);
                    break;
                } else if leading_whitespaces + 2 <= nesting_spaces[nd] {
                    // Sublist level has decreased
                    nesting_depth -= 1;
                    nesting_spaces.pop();
                    continue;
                } else {
                    break;
                }
            }
            match values.pop() {
                Some(Value::Notation(notation, mut values_depths)) => {
                    values_depths = values_depths.modify_sub(nesting_depth, sublist, line);
                    values.push(Value::Notation(notation.clone(), values_depths));
                }
                Some(Value::Text(mut values_depths)) => {
                    values_depths = values_depths.modify_sub(nesting_depth, sublist, line);
                    values.push(Value::Text(values_depths));
                }
                None => {
                    values.push(Value::Text(NestedString::new(line)));
                    nesting_depth = 0;
                    nesting_spaces = vec![0];
                }
                Some(v) => {
                    values.push(v);
                    values.push(Value::Text(NestedString::new(line)));
                    nesting_depth = 0;
                    nesting_spaces = vec![0];
                }
            }
        } else {
            values.push(value);
            nesting_depth = 0;
            nesting_spaces = vec![0];
        }
    }
    values.push(Value::Separator);

    values
}

/// The enum used to represent values of a _raw_ bindgen file
#[derive(Clone, Debug)]
pub enum StringType {
    /// Parsed value
    Parsed(Vec<Value>),
    /// No-doc, value is given raw
    Raw(String),
}

/// Generate a [`Vec`] of [`StringType`] from a given [`&str`], assuming it's a _raw_ bindgen file
pub fn parse_bindgen(input: &str) -> Vec<StringType> {
    let lines: Vec<String> = input
        .split('\n')
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    let mut strings = vec![];

    let mut comment_buffer = vec![];
    for line in lines {
        if line.trim().starts_with("#[doc = \"") && line.trim().ends_with("\"]") {
            comment_buffer.push(line.replace("#[doc = \"", "").replace("\"]", ""));
        } else {
            if !comment_buffer.is_empty() {
                strings.push(StringType::Parsed(parse_comment(
                    comment_buffer.join("\n").as_str(),
                )));
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
    use crate::parser::NestedString;
    use crate::parser::Value::Notation;

    #[test]
    fn test() {
        let parsed = parse_comment("@param random Random thing lmao\n\n@block This is going to be\nA block of text\nThis is crazy right??\n\nHello this is not anotated\n");
        println!("{:?}", parsed);
    }

    #[test]
    fn italic_works() {
        let parsed = parse_comment("@brief \\a example \\\\e example 2 @em example 3");
        assert_eq!(
            parsed[0],
            Notation(
                "@brief".to_owned(),
                NestedString {
                    top: "*example* *example* 2 *example* 3".to_owned(),
                    sub: vec![]
                }
            )
        )
    }

    #[test]
    fn emojis_work() {
        let parsed = parse_comment("@brief @emoji :smirk: \\emoji smirk \\\\emoji smiley");
        assert_eq!(
            parsed[0],
            Notation(
                "@brief".to_owned(),
                NestedString {
                    top: "😏 😏 😃".to_owned(),
                    sub: vec![]
                }
            )
        )
    }
}
