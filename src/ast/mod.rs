//! This module contains functions that generate ASTs from [`Vec`] of [`crate::parser::Value`]
//!
//! **The functions and structs here should _not_ be considered stable**

use std::fmt::{Display, Formatter};
use crate::parser::{Value, NestedString};
use crate::utils::NotationMatching;

/// Represents a parsed Doxygen comment.
#[derive(Clone, Debug)]
pub struct ParsedDoxygen {
    /// _Title_ of the item being documented
    pub title: Option<NestedString>,
    /// A _brief_ introduction to the item being documented.
    pub brief: Option<NestedString>,
    /// A _description_ of the item being documented.
    pub description: Option<Vec<NestedString>>,
    /// List of _warnings_ of the item being documented.
    pub warnings: Option<Vec<Warning>>,
    /// List of _notes_ of the item being documented.
    pub notes: Option<Vec<Note>>,
    /// The _parameters_ of the item being documented.
    pub params: Option<Vec<Param>>,
    /// Data about the _deprecation_ status of the item being documented.
    pub deprecated: Option<Deprecated>,
    /// List of _To Do_s of the item being documented.
    pub todos: Option<Vec<NestedString>>,
    /// Description of the values being _returned_
    pub returns: Option<Vec<Return>>,
    /// Possible _return values_ of the item being documented
    pub return_values: Option<Vec<ReturnValue>>,
}

/// Represents a single _parameter_.
#[derive(Clone, Debug)]
pub struct Param {
    /// The _argument name_ of the item being documented.
    pub arg_name: String,
    /// The _direction_ of the argument. [Check Doxygen docs for more info](https://www.doxygen.nl/manual/commands.html#cmdparam)
    pub direction: Option<Direction>,
    /// The _description_ of the argument.
    pub description: Option<NestedString>,
}

/// Represents the _deprecation_ status of an item.
#[derive(Clone, Debug)]
pub struct Deprecated {
    /// Whether _is deprecated_ or not
    pub is_deprecated: bool,
    /// The message left with the deprecation status
    pub message: Option<NestedString>,
}

/// Represents a _note_
#[derive(Clone, Debug)]
pub struct Note(pub NestedString);

/// Represents a _warning_
#[derive(Clone, Debug)]
pub struct Warning(pub NestedString);

/// Represents a _return_
#[derive(Clone, Debug)]
pub struct Return(pub NestedString);

/// Represents a _return_ value
#[derive(Clone, Debug)]
pub struct ReturnValue(pub NestedString);

/// The _direction_ of an argument. [Check Doxygen docs for more info](https://www.doxygen.nl/manual/commands.html#cmdparam)
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Direction {
    In,
    Out,
    InOut,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::In => f.write_str("In"),
            Direction::Out => f.write_str("In, Out"),
            Direction::InOut => f.write_str("Out"),
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "in" {
            Ok(Direction::In)
        } else if value == "out" {
            Ok(Direction::Out)
        } else if value == "in,out" || value == "out,in" {
            Ok(Direction::InOut)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone)]
enum MoveBufferTo {
    Description,
    ToDo,
}

/// Generates a [`ParsedDoxygen`] from a [`Vec`] of [`crate::parser::Value`]
///
/// # Examples
/// ```
/// use doxygen_rs::ast::generate_ast;
/// use doxygen_rs::parser::parse_comment;
///
/// let parsed = parse_comment("@brief Random comment");
/// let ast = generate_ast(parsed);
/// ```
pub fn generate_ast(input: Vec<Value>) -> ParsedDoxygen {
    let mut title = None;
    let mut brief = None;
    let mut deprecated = None;
    let mut return_values = vec![];
    let mut returns = vec![];
    let mut warnings = vec![];
    let mut notes = vec![];
    let mut todos = vec![];
    let mut params = vec![];
    let mut description = vec![];

    let mut currently_saving_paragraph = false;
    let mut paragraph_buffer = vec![];
    let mut move_buffer_to = None;

    for value in input {
        match value {
            Value::Notation(notation, mut content) => {
                if notation.starts_with_notation("brief") {
                    brief = Some(content);
                } else if notation.starts_with_notation("deprecated") {
                    deprecated = Some(Deprecated {
                        is_deprecated: true,
                        message: if content.top.is_empty() { None } else { Some(content) },
                    });
                } else if notation.starts_with_notation("details") {
                    currently_saving_paragraph = true;
                    move_buffer_to = Some(MoveBufferTo::Description);
                    paragraph_buffer.push(content);
                } else if notation.starts_with_notation("todo") {
                    currently_saving_paragraph = true;
                    move_buffer_to = Some(MoveBufferTo::ToDo);
                    paragraph_buffer.push(content);
                } else if notation.starts_with_notation("param") {
                    let direction = {
                        let raw_direction = notation.remove_notation("param");
                        if raw_direction.is_empty() || raw_direction.starts_with("[]") {
                            None
                        } else if raw_direction.starts_with("[in]") {
                            Some(Direction::In)
                        } else if raw_direction.starts_with("[out]") {
                            Some(Direction::Out)
                        } else if raw_direction.starts_with("[in,out]") || raw_direction.starts_with("[out,in]") {
                            Some(Direction::InOut)
                        } else {
                            None
                        }
                    };

                    let arg_name = {
                        let split = content.top.split_whitespace().map(|v| v.to_string()).collect::<Vec<String>>();
                        split.first().unwrap().to_owned()
                    };

                    let description = {
                        let split: Vec<&str> = content.top.splitn(2, char::is_whitespace).collect();
                        content.top = split[1].to_string();
                        if content.top.is_empty() {
                            None
                        } else {
                            Some(content)
                        }
                    };

                    params.push(Param {
                        arg_name,
                        direction,
                        description,
                    })
                } else if notation.starts_with_notation("return") || notation.starts_with_notation("returns") {
                    returns.push(Return(content));
                } else if notation.starts_with_notation("name") {
                    title = Some(content)
                } else if notation.starts_with_notation("note") || notation.starts_with_notation("remark") || notation.starts_with_notation("remarks") {
                    notes.push(Note(content))
                } else if notation.starts_with_notation("warning") {
                    warnings.push(Warning(content))
                } else if notation.starts_with_notation("retval") {
                    return_values.push(ReturnValue(content))
                }
            }
            Value::Text(content) => {
                if currently_saving_paragraph {
                    paragraph_buffer.push(content);
                } else if content.to_string().trim() != "*" && content.to_string().trim() != "*/" && content.to_string().trim() != "**" {
                    description.push(content);
                }
            }
            Value::Separator => {
                if currently_saving_paragraph {
                    if let Some(move_buffer_to) = move_buffer_to.clone() {
                        match move_buffer_to {
                            MoveBufferTo::Description => description.append(&mut paragraph_buffer.clone()),
                            MoveBufferTo::ToDo => todos.append(&mut paragraph_buffer.clone()),
                        }
                    }
                    currently_saving_paragraph = false;
                    paragraph_buffer = vec![];
                    move_buffer_to = None;
                }
            }
            Value::Unknown => {}
            Value::Continuation(_, _, _) => unreachable!(),//only for parser internal use 
        }
    }

    // TODO: Improve this
    let description = (!description.is_empty()).then_some(description);
    let returns = (!returns.is_empty()).then_some(returns);
    let todos = (!todos.is_empty()).then_some(todos);
    let warnings = (!warnings.is_empty()).then_some(warnings);
    let notes = (!notes.is_empty()).then_some(notes);
    let params = (!params.is_empty()).then_some(params);
    let return_values = (!return_values.is_empty()).then_some(return_values);

    ParsedDoxygen {
        title,
        brief,
        description,
        warnings,
        notes,
        params,
        deprecated,
        todos,
        returns,
        return_values,
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use super::*;

    #[test]
    fn parses_param() {
        let doxygen = generate_ast(parser::parse_comment("@param random Random thing lmao\n@param[in] goes_in This goes in lmao"));

        let first_param = doxygen.params.as_ref().unwrap();
        let first_param = first_param.get(0).unwrap();
        assert_eq!(first_param.arg_name, "random");
        assert_eq!(first_param.description, Some(NestedString::new("Random thing lmao".to_string())));
        assert_eq!(first_param.direction, None);

        let second_param = doxygen.params.as_ref().unwrap();
        let second_param = second_param.get(1).unwrap();
        assert_eq!(second_param.arg_name, "goes_in");
        assert_eq!(second_param.description, Some(NestedString::new("This goes in lmao".to_string())));
        assert_eq!(second_param.direction, Some(Direction::In));
    }

    #[test]
    fn parses_brief() {
        let doxygen = generate_ast(parser::parse_comment("@brief This function does things"));

        assert_eq!(doxygen.brief, Some(NestedString::new("This function does things".to_string())));
    }

    #[test]
    fn parses_description() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is a function\n\nThis is the description of the thing.\nYou should do things with this function.\nOr not, I don't really care."));

        assert_eq!(doxygen.description, Some(vec![NestedString::new("This is the description of the thing.".to_string()), NestedString::new("You should do things with this function.".to_string()),NestedString::new("Or not, I don't really care.".to_string())]))
    }
    #[test]
    fn parses_multiline_simple() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is a function\n\nThis is the description of the thing.\n    You should do things with this function.\n    Or not, I don't really care."));

        assert_eq!(doxygen.description, Some(vec![NestedString::new("This is the description of the thing. You should do things with this function. Or not, I don't really care.".to_string())]))
    }
    #[test]
    fn parses_multiline_sublist() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is a function\n\nThis is the description of the thing.\n    - You should do things with this function.\n    - Or not, I don't really care."));

        assert_eq!(doxygen.description, Some(
            vec![
                NestedString{
                    top:"This is the description of the thing.".to_string(),
                    sub: vec![
                        NestedString::new("You should do things with this function.".to_string()),
                        NestedString::new("Or not, I don't really care.".to_string())]}]))
    }
    #[test]
    fn parses_multiline_sublist_complex() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is a function\n\nThis is the description of the thing.\n    - You should do things with:\n      - this function.\n      - that function.\n        (with long description)\n    - Or not, I don't really care."));

        assert_eq!(doxygen.description, Some(
            vec![
                NestedString{
                    top:"This is the description of the thing.".to_string(),
                    sub: vec![
                        NestedString{
                            top:"You should do things with:".to_string(),
                            sub: vec![
                                NestedString::new("this function.".to_string()),
                                NestedString::new("that function. (with long description)".to_string())]},
                        NestedString::new("Or not, I don't really care.".to_string())]}]));
    }
    #[test]
    fn parses_deprecated() {
        let doxygen = generate_ast(parser::parse_comment("@deprecated This function is pure spaghetti lmao\n\n@brief Creates a single spaghetti"));

        let deprecated = doxygen.deprecated.unwrap();
        assert_eq!(deprecated.is_deprecated, true);
        assert_eq!(deprecated.message, Some(NestedString::new("This function is pure spaghetti lmao".to_string())));
    }

    #[test]
    fn parses_details() {
        let doxygen = generate_ast(parser::parse_comment("@brief This does things\n\n@details This does _advanced_ things\nAnd the _advanced_ things are not easy"));

        let description = doxygen.description.unwrap();
        assert_eq!(description, vec![NestedString::new("This does _advanced_ things".to_string()), NestedString::new("And the _advanced_ things are not easy".to_string())]);
    }

    #[test]
    fn parses_todo() {
        let doxygen = generate_ast(parser::parse_comment("@brief This is WIP\n\n@todo Fix the bug where the C: drive is deleted"));

        let todos = doxygen.todos.unwrap();
        assert_eq!(todos.get(0).unwrap().clone(), NestedString::new("Fix the bug where the C: drive is deleted".to_string()));
    }

    #[test]
    fn parses_advanced_doxygen() {
        let doxygen = generate_ast(parser::parse_comment("@brief Creates a new dog.\n\nCreates a new Dog named `_name` with half of its maximum energy.\n\n@param _name The dog's name."));

        let first_param = doxygen.params.unwrap();
        let first_param = first_param.first().unwrap();
        assert_eq!(doxygen.brief, Some(NestedString::new("Creates a new dog.".to_string())));
        assert_eq!(doxygen.description, Some(vec![NestedString::new("Creates a new Dog named `_name` with half of its maximum energy.".to_string())]));
        assert_eq!(first_param.arg_name, "_name".to_string());
        assert_eq!(first_param.description, Some(NestedString::new("The dog's name.".to_string())));
        assert_eq!(first_param.direction, None);
    }
}